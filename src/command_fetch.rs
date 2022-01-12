
use {
    crate::{Anyhow, Options},
    podchamp::{Database, GetFeeds, models::Feed, Episode, EpisodeMeta, FeedIndex, plan_fetch},
    anyhow::bail,
    chrono::prelude::*,
    futures::{
        stream::FuturesUnordered,
        StreamExt as _,
    },
    url::Url,
};

async fn fetch_feed(
    join_result: Result<(Feed<'_>, reqwest::Result<bytes::Bytes>), tokio::task::JoinError>,
    db: &mut Database,
    now: DateTime<Utc>,
    opts: &Options,
) -> Anyhow<u32> {
    // handle and parse response
    let (feed, fetch_result) = join_result?;
    let bytes = fetch_result?;
    let raw_index = feed_rs::parser::parse(&bytes[..])?;
    let index = parse_index(&raw_index, now);
    if index.is_empty() {
        bail!("{} contains no recognizable episodes", &feed.name);
    }

    // fetch logic
    let plan = plan_fetch(&feed, &index);
    if let Some(threshold) = plan.set_fetch_since {
        db.set_fetch_since(&feed.name, &threshold)?;
    }

    let mut n_fetched = 0;
    for Episode{meta, id, url, when} in plan.episodes {
        // TODO do this in one go for all newest items
        if !db.is_episode_registered(&feed.name, id)? {
            start_download(&opts, &feed, meta, url, when).await?;
            n_fetched += 1;
            db.register_episode(&feed.name, id)?;
        }
    }

    Ok(n_fetched)
}

pub(crate)
async fn fetch<'a, 'db> (
    db:   &'db mut Database,
    feed: Option<&'_ str>,
    opts: &'a Options,
    now:  DateTime<Utc>,
) -> Anyhow<()> {
    // figure out what to fetch
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

    // fetch feed data, supplying responses as they come in
    let web_client = reqwest::Client::new();
    let mut jobs = feeds.into_iter()
        .map(|feed| {
            let request = web_client.get(feed.uri.as_ref()).build().unwrap();
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

    // perform per-feed fetches
    let mut n_fetched = 0;
    while let Some(join_result) = jobs.next().await {
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

fn parse_index<'a> (index: &'a feed_rs::model::Feed, now: DateTime<Utc>)
    -> FeedIndex<'a>
{
    index.entries.iter()
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
            let title = entry.title.as_ref().map(|title| &title.content[..]);
            let meta = EpisodeMeta{title};
            let id = &entry.id;
            Some(Episode{meta, id, url, when})
        })
        // ignore time-travellers
        .filter(|ep| ep.when < now)
        .collect()
}

async fn start_download(
    opts: &Options,
    feed: &Feed<'_>,
    meta: &EpisodeMeta<'_>,
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
        ("PODCHAMP_TITLE",       meta.title),
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

