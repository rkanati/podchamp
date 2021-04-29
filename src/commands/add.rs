
use {
    crate::{Anyhow, schema, Db, models},
    std::num::NonZeroU32,
    anyhow::anyhow,
    url::Url,
};

pub(crate)
async fn add(db: &Db, name: &str, link: &Url, backlog: Option<NonZeroU32>) -> Anyhow<()> {
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
        .execute(db)
        .map_err(|e| match e {
            Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _)
                => anyhow!("{} already exists", name),
            e   => anyhow!(e)
        })?;

    eprintln!("Added {}", name);
    Ok(())
}

