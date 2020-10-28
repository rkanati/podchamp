
#![feature(try_blocks)]

#[macro_use]
extern crate diesel;

#[macro_use]
extern crate diesel_migrations;

embed_migrations!();

mod schema;
mod models;
mod options;

use {
    crate::options::*,
    anyhow::{anyhow, bail},
    chrono::prelude::*,
};

fn open_db(path: &std::ffi::OsStr) -> diesel::sqlite::SqliteConnection {
    use diesel::prelude::*;
    let path = path.clone().to_str().unwrap();
    let db = SqliteConnection::establish(path)
        .expect("Error opening feeds database");
    embedded_migrations::run(&db).unwrap();
    db
}

async fn start_download(
    opts: &Options,
    feed: &models::Feed,
    item: &rss::Item,
    date: &DateTime<Utc>)
    -> anyhow::Result<()>
{
    let uri = item.enclosure().unwrap().url();

    let mut command = tokio::process::Command::new(&opts.downloader);
    command.arg(uri);

    let date = date.format(&opts.date_format)
        .to_string();

    let envs = [
        ("PODCHAMP_FEED",        Some(&feed.name[..])),
        ("PODCHAMP_DATE",        Some(&date[..])),
        ("PODCHAMP_TITLE",       item.title()),
        ("PODCHAMP_AUTHOR",      item.author()),
        ("PODCHAMP_DESCRIPTION", item.description()),
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let now = Utc::now();

    dotenv::dotenv().ok();
    let opts = options::Options::parse();
    let db = open_db(&opts.database_path);

    match &opts.command {
        Command::Add{name, link, backlog} => {
            let backlog = backlog.map(|n| n.get()).unwrap_or(1);

            let feed = models::NewFeed {
                name,
                uri: link.as_str(),
                backlog: backlog as i32,
                fetch_since: None
            };

            use diesel::{prelude::*, result::{Error, DatabaseErrorKind}};
            diesel::insert_into(schema::feeds::table)
                .values(&feed)
                .execute(&db)
                .map_err(|e| match e {
                    Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _)
                        => anyhow!("{} already exists", name),
                    e   => anyhow!(e)
                })?;

            eprintln!("Added {}", name);
        }

        Command::Rm{name} => {
            use{diesel::prelude::*, schema::feeds::dsl as dsl};
            let n = diesel::delete(dsl::feeds.filter(dsl::name.eq(name)))
                .execute(&db)?;
            if n == 0 {
                eprintln!("{} is not a feed", name);
            }
        }

        Command::Ls => {
            use{diesel::prelude::*, schema::feeds::dsl as dsl};
            let results = dsl::feeds
                .load::<models::Feed>(&db)?;
            if results.is_empty() {
                eprintln!("No feeds. You can add one with `podchamp add`.");
            }
            else {
                for feed in results {
                    println!("{} @{}", feed.name, feed.uri);
                }
            }
        }

        Command::Mod{feed, how} => {
            match how {
                Modification::Link{link} => {
                    use{diesel::prelude::*, schema::feeds::dsl as dsl};
                    let n = diesel::update(dsl::feeds.filter(dsl::name.eq(feed)))
                        .set(dsl::uri.eq(link.as_str()))
                        .execute(&db)?;
                    if n == 0 { bail!("{} is not a feed", feed); }
                    eprintln!("Changed {} RSS link to {}", feed, link);
                }

                Modification::Backlog{n} => {
                    use{diesel::prelude::*, schema::feeds::dsl as dsl};
                    let n = diesel::update(dsl::feeds.filter(dsl::name.eq(feed)))
                        .set(dsl::backlog.eq(n.get() as i32))
                        .execute(&db)?;
                    if n == 0 { bail!("{} is not a feed", feed); }
                    eprintln!("Changed {} backlog to {}", feed, n);
                }
            }
        }

        Command::Reset{feed} => {
            use diesel::prelude::*;

            {
                use schema::register::dsl as dsl;
                diesel::delete(dsl::register.filter(dsl::feed.eq(feed)))
                    .execute(&db)?;
            }

            {
                use schema::feeds::dsl as dsl;
                diesel::update(dsl::feeds.filter(dsl::name.eq(feed)))
                    .set(dsl::fetch_since.eq::<Option<NaiveDateTime>>(None))
                    .execute(&db)?;
            }

            eprintln!("Progress reset for {}", feed);
        }

        Command::Fetch{feed} => {
            let feeds = {
                use {diesel::prelude::*, schema::feeds::dsl::*};
                let query =
                    if let Some(feed) = feed { feeds.filter(name.eq(feed)).into_boxed() }
                    else                     { feeds.into_boxed() };
                query.load::<models::Feed>(&db)?
            };

            let client = reqwest::Client::new();

            for feed in feeds.iter() {
                eprintln!("Fetching {}", feed.name);

                let req = client.get(&feed.uri).build().unwrap();
                let fetch_since: Option<DateTime<Utc>> = feed
                    .fetch_since
                    .map(|naive| DateTime::from_utc(naive, Utc));
                    //.unwrap_or(chrono::MIN_DATETIME);

                let result: anyhow::Result<_> = try {
                    let resp = client.execute(req).await?;
                    let feed_xml = resp.bytes().await?;
                    let channel = rss::Channel::read_from(&feed_xml[..])?;

                    // build a list of most-recent episodes
                    let mut recents: Vec<_> = channel.items().iter()
                        // ignore items with no actual episode to download
                        .filter(|item| item.enclosure().is_some())
                        // parse item dates, ignoring items with no date
                        .filter_map(|item| item
                            .pub_date()
                            .and_then(|date| DateTime::parse_from_rfc2822(date).ok())
                            .map(|date| {
                                let date: DateTime<Utc> = date.into();
                                (item, date)
                            })
                        )
                        // ignore time-travellers
                        .filter(|(_, date)| date < &now)
                        // parse GUIDs, ignoring items with none
                        .filter_map(|(item, date)| item
                            .guid()
                            .map(|guid| (item, date, guid.value.to_owned()))
                        )
                        .collect();

                    // sort the list by descending date
                    recents.sort_unstable_by_key(|(_, date, _)| std::cmp::Reverse(*date));

                    let backlog_start_index = (feed.backlog as usize).max(1).min(recents.len()) - 1;
                    let (_, backlog_start_date, _) = recents[backlog_start_index];

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
                            .execute(&db)?;
                    };

                    let to_fetch = recents.iter()
                        .filter(|(_, date, _)| date >= &threshold);

                    for (item, date, guid) in to_fetch {
                        // TODO do this in one go for all newest items
                        let already_got = {
                            use {diesel::prelude::*, schema::register::dsl};
                            let n: i64 = dsl::register
                                .filter(dsl::feed.eq(&feed.name))
                                .filter(dsl::guid.eq(&guid))
                                .count()
                                .get_result(&db)?;
                            n != 0
                        };

                        if already_got { continue; }

                        start_download(&opts, &feed, &item, &date).await?;

                        let registration = models::NewRegistration {
                            feed: &feed.name,
                            guid: &guid
                        };
                        use diesel::prelude::*;
                        diesel::insert_into(schema::register::table)
                            .values(&registration)
                            .execute(&db)?;
                    }
                };

                if let Err(e) = result {
                    eprintln!("Error fetching {}: {}", feed.name, e);
                }
            }
        }
    }

    Ok(())
}

