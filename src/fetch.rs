
use {
    crate::{Anyhow, Options},
    podchamp::{Database, GetFeeds, models::Feed},
    anyhow::bail,
    chrono::prelude::*,
    futures::{
        stream::FuturesUnordered,
        StreamExt as _,
    },
    url::Url,
};

struct RecentEpisode<'c> {
    entry: &'c feed_rs::model::Entry,
    url: &'c Url,
    when: DateTime<Utc>,
}

fn fetch_threshold(feed: &Feed, recents: &[RecentEpisode<'_>])
    -> (DateTime<Utc>, bool)
{
    // find date of first episode within backlog
    let backlog_start_index = (feed.backlog as usize).max(1).min(recents.len()) - 1;
    let backlog_start_date = recents[backlog_start_index].when;

    // figure out what date to fetch back to
    if let Some(since) = feed
        .fetch_since
        .map(|naive| DateTime::from_utc(naive, Utc))
        .filter(|since| since <= &backlog_start_date)
    {
        // mature feed - keep fetching from the established date
        (since, false)
    }
    else {
        // new feed, or backlog increased back past since-date - fetch from start of
        // backlog
        (backlog_start_date, true)
    }
}

async fn fetch_feed(
    join_result: Result<(Feed, reqwest::Result<bytes::Bytes>), tokio::task::JoinError>,
    db: &mut Database,
    now: DateTime<Utc>,
    opts: &Options,
) -> Anyhow<u32> {
    let (feed, fetch_result) = join_result?;
    let bytes = fetch_result?;
    let channel = feed_rs::parser::parse(&bytes[..])?;

    // build a list of most-recent episodes
    let recents = collect_recent_episodes(&channel, &now);
    if recents.is_empty() {
        bail!("{} contains no recognizable episodes", &feed.name);
    }

    // figure out how far back to fetch
    let (threshold, update_db) = fetch_threshold(&feed, &recents);
    if update_db {
        db.set_fetch_since(&feed.name, &threshold)?;
    }

    let mut n_fetched = 0;
    for RecentEpisode{entry, url, when} in recents.iter()
        .take_while(|recent| recent.when >= threshold)
    {
        // TODO do this in one go for all newest items
        if !db.is_episode_registered(&feed.name, &entry.id)? {
            start_download(&opts, &feed, &entry, &url, &when).await?;
            n_fetched += 1;
            db.register_episode(&feed.name, &entry.id)?;
        }
    }

    Ok(n_fetched)
}

pub(crate)
async fn fetch<'a> (
    db:   &'a mut Database,
    feed: Option<&'_ str>,
    opts: &'a Options,
    now:  DateTime<Utc>,
) -> Anyhow<()> {
    let feeds = db.get_feeds(match feed {
        None       => GetFeeds::All,
        Some(feed) => GetFeeds::One(feed)
    })?;

    if feeds.is_empty() {
        eprintln!("No feeds. You can add one with `podchamp add`.");
        return Ok(())
    }

    eprint!("Fetching {}", &feeds[0].name);
    for feed in &feeds[1..] {
        eprint!(", {}", &feed.name);
    }
    eprintln!();

    let web_client = reqwest::Client::new();
    let mut channels = feeds.into_iter()
        .map(|feed| {
            let request = web_client.get(&feed.uri).build().unwrap();
            let web_client = web_client.clone();
            tokio::spawn(async move {
                let resp = match web_client.execute(request).await {
                    Ok(resp) => resp,
                    Err(e) => return (feed, Anyhow::from(Err(e)))
                };
                let result = resp.bytes().await;
                (feed, Anyhow::from(result))
            })
        })
        .collect::<FuturesUnordered<_>>();

    let mut n_fetched = 0;
    while let Some(join_result) = channels.next().await {
        match fetch_feed(join_result, db, now, opts).await {
            Ok(n)  => { n_fetched += n; }
            Err(e) => { eprintln!("Fetch error: {}", e); }
        }
    }

    if n_fetched == 0 {
        eprintln!("Already up-to-date");
    }

    Ok(())
}

fn collect_recent_episodes<'c> (channel: &'c feed_rs::model::Feed, now: &DateTime<Utc>)
    -> Vec<RecentEpisode<'c>>
{
    let mut recents: Vec<_> = channel.entries.iter()
        // ignore items with no date, or no actual episode to download
        .filter_map(|entry| {
            let when = entry.published?;
            // TODO sort this out. as of feed-rs 0.6, rss enclosures are emulated with
            // mediarss media objects, but this is very janky and not really consistent
            // with podcasts as they are normally understood. file a bug? not sure.
            let url = entry.media.iter()
                .flat_map(|media_obj| media_obj.content.iter())
                .find_map(|content| {
                    let mime = content.content_type.as_ref()?;
                    if mime.type_() != "audio" { return None; }
                    content.url.as_ref()
                })?;
            Some(RecentEpisode{entry, url, when})
        })
        // ignore time-travellers
        .filter(|recent| &recent.when < now)
        .collect();

    // sort the list by descending date
    recents.sort_unstable_by_key(|recent| std::cmp::Reverse(recent.when));

    recents
}

async fn start_download(
    opts: &Options,
    feed: &Feed,
    item: &feed_rs::model::Entry,
    link: &Url,
    date: &DateTime<Utc>)
    -> Anyhow<()>
{
    let mut command = tokio::process::Command::new(&opts.downloader);
    command.arg(link.as_str());

    let date = date.format(&opts.date_format)
        .to_string();

    let envs = [
        ("PODCHAMP_FEED",        Some(&feed.name[..])),
        ("PODCHAMP_DATE",        Some(&date[..])),
        ("PODCHAMP_TITLE",       item.title.as_ref().map(|title| &title.content[..])),
    //  ("PODCHAMP_AUTHOR",      item.author()),
    //  ("PODCHAMP_DESCRIPTION", item.summary),
    ];

    for (var, value) in envs.iter() {
        if let Some(value) = value {
            command.env(var, value);
        }
    }

    let child = command.spawn()?;
    let output = child.wait_with_output().await?;
    if !output.status.success() {
        bail!("Download command failed with code {:?}", output.status.code());
    }

    Ok(())
}

