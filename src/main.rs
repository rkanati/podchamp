
#![forbid(unsafe_code)]
#![feature(try_blocks)]

#[macro_use]
extern crate diesel;

#[macro_use]
extern crate diesel_migrations;

embed_migrations!();

mod commands;
mod models;
mod options;
mod schema;

use {
    crate::options::*,
    anyhow::{anyhow, bail},
    chrono::prelude::*,
    url::Url,
};

pub(crate) use diesel::sqlite::SqliteConnection as Db;

fn open_db(path: &std::path::Path) -> anyhow::Result<Db> {
    let bad_path_error = || anyhow!("Invalid database path");

    let dir = path.parent().ok_or_else(bad_path_error)?;
    std::fs::create_dir_all(dir)?;

    let path = path.to_str().ok_or_else(bad_path_error)?;
    use diesel::prelude::*;
    let db = SqliteConnection::establish(path)?;
    embedded_migrations::run(&db)?;

    Ok(db)
}


#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let now = Utc::now();

    let opts = options::Options::load();
    let db = open_db(&opts.database_path)?;

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
                    // TODO make an effort to tabulate
                    println!("{:16} {}", feed.name, feed.uri);
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
                    eprintln!("Changed {} feed link to {}", feed, link);
                }

                Modification::Backlog{n} => {
                    use{diesel::prelude::*, schema::feeds::dsl as dsl};
                    let n_updated = diesel::update(dsl::feeds.filter(dsl::name.eq(feed)))
                        .set(dsl::backlog.eq(n.get() as i32))
                        .execute(&db)?;
                    if n_updated == 0 { bail!("{} is not a feed", feed); }
                    eprintln!("Changed {} backlog to {}", feed, n);
                }
            }
        }

        Command::Reset{feed} => commands::reset(&db, feed).await?,
        Command::Fetch{feed} => commands::fetch(&opts, &db, feed.as_deref()).await?,
    }

    Ok(())
}

