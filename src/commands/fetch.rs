
use {
    crate::{Anyhow, schema, models, Db, Options},
    anyhow::bail,
    chrono::prelude::*,
    futures::{
        stream::FuturesUnordered,
        StreamExt as _,
    },
    url::Url,
};

pub(crate)
async fn fetch<'a> (
    now:  DateTime<Utc>,
    opts: &'a Options,
    db:   &'a Db,
    feed: Option<&'_ str>)
    -> Anyhow<()>
{
    let feeds = {
        use {diesel::prelude::*, schema::feeds::dsl::*};
        let query =
            if let Some(feed) = feed { feeds.filter(name.eq(feed)).into_boxed() }
            else                     { feeds.into_boxed() };
        query.load::<models::Feed>(db)?
    };

    if feeds.is_empty() {
        eprintln!("No feeds. You can add one with `podchamp add`.");
        return Ok(())
    }

    let web_client = reqwest::Client::new();
    let mut channels = feeds.into_iter()
        .map(|feed| {
            let request = web_client.get(&feed.uri).build().unwrap();
            let web_client = web_client.clone();
            eprintln!("Fetching {}", &feed.name);
            tokio::spawn(async move {
                let result: Anyhow<_> = try {
                    web_client.execute(request).await?
                        .bytes().await?
                };
                (feed, result)
            })
        })
        .collect::<FuturesUnordered<_>>();

    while let Some(join_result) = channels.next().await {
        let result: Anyhow<_> = try {
            let (feed, fetch_result) = join_result?;
            let bytes = fetch_result?;

            let channel = feed_rs::parser::parse(&bytes[..])?;

            let fetch_since: Option<DateTime<Utc>> = feed
                .fetch_since
                .map(|naive| DateTime::from_utc(naive, Utc));

            // build a list of most-recent episodes
            let mut recents: Vec<_> = channel.entries.iter()
                // ignore items with no date, or no actual episode to download
                .filter_map(|item| {
                    let date = item.published?;
                    // TODO sort this out. as of feed-rs 0.6, rss enclosures are emulated with
                    // mediarss media objects, but this is very janky and not really consistent
                    // with podcasts as they are normally understood. file a bug? not sure.
                    let link = item.media.iter()
                        .flat_map(|media_obj| media_obj.content.iter())
                        .filter_map(|content| {
                            let mime = content.content_type.as_ref()?;
                            if mime.type_() != "audio" { return None; }
                            content.url.as_ref()
                        })
                        .next()?;
                    Some((item, link, date))
                })
                // ignore time-travellers
                .filter(|(_, _, date)| date < &now)
                .collect();

            if recents.is_empty() {
                eprintln!("{} contains no recognizable episodes", &feed.name);
                continue;
            }

            // sort the list by descending date
            recents.sort_unstable_by_key(|(_, _, date)| std::cmp::Reverse(*date));

            let backlog_start_index = (feed.backlog as usize).max(1).min(recents.len()) - 1;
            let (_, _, backlog_start_date) = recents[backlog_start_index];

            let (threshold, set_fetch_since) = if let Some(since) = fetch_since {
                if since <= backlog_start_date { (since,              false) }
                else                           { (backlog_start_date, true ) }
            }
            else {
                (backlog_start_date, true)
            };

            if set_fetch_since {
                use{diesel::prelude::*, schema::feeds::dsl as dsl};
                diesel::update(dsl::feeds.filter(dsl::name.eq(&feed.name)))
                    .set(dsl::fetch_since.eq(Some(threshold.naive_utc())))
                    .execute(db)?;
            };

            let to_fetch = recents.iter()
                .filter(|(_, _, date)| date >= &threshold);

            for (item, link, date) in to_fetch {
                // TODO do this in one go for all newest items
                let already_got = {
                    use {diesel::prelude::*, schema::register::dsl};
                    let n: i64 = dsl::register
                        .filter(dsl::feed.eq(&feed.name))
                        .filter(dsl::guid.eq(&item.id))
                        .count()
                        .get_result(db)?;
                    n != 0
                };

                if already_got { continue; }

                start_download(&opts, &feed, &item, &link, &date).await?;

                let registration = models::NewRegistration {
                    feed: &feed.name,
                    guid: &item.id
                };
                use diesel::prelude::*;
                diesel::insert_into(schema::register::table)
                    .values(&registration)
                    .execute(db)?;
            }
        };

        if let Err(e) = result { println!("Fetch error: {}", e); }
    }

    Ok(())
}

async fn start_download(
    opts: &Options,
    feed: &models::Feed,
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

