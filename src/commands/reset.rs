
use {
    crate::{schema, Db},
    chrono::prelude::*,
};

pub(crate)
async fn reset(db: &Db, feed: Option<&str>) -> anyhow::Result<()> {
    if feed.is_none() {
        todo!("reset for all feeds");
    }

    use diesel::prelude::*;

    {
        use schema::register::dsl as dsl;
        diesel::delete(dsl::register.filter(dsl::feed.eq(feed.unwrap())))
            .execute(db)?;
    }

    {
        use schema::feeds::dsl as dsl;
        diesel::update(dsl::feeds.filter(dsl::name.eq(feed.unwrap())))
            .set(dsl::fetch_since.eq::<Option<NaiveDateTime>>(None))
            .execute(db)?;
    }

    eprintln!("Progress reset for {}", feed.unwrap_or("all feeds"));
    Ok(())
}

