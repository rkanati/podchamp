
pub(crate)
async fn reset(db: &Db, feed: Option<&str>) -> anyhow::Result<()> {
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

