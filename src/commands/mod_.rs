
use {
    crate::{Anyhow, Db, options::Modification},
    podchamp::schema,
    anyhow::bail,
};

pub(crate)
async fn mod_(db: &Db, feed: &str, how: &Modification) -> Anyhow<()> {
    match how {
        Modification::Link{link} => {
            use{diesel::prelude::*, schema::feeds::dsl as dsl};
            let n = diesel::update(dsl::feeds.filter(dsl::name.eq(feed)))
                .set(dsl::uri.eq(link.as_str()))
                .execute(db)?;
            if n == 0 { bail!("{} is not a feed", feed); }
            eprintln!("Changed {} feed link to {}", feed, link);
        }

        Modification::Backlog{n} => {
            use{diesel::prelude::*, schema::feeds::dsl as dsl};
            let n_updated = diesel::update(dsl::feeds.filter(dsl::name.eq(feed)))
                .set(dsl::backlog.eq(n.get() as i32))
                .execute(db)?;
            if n_updated == 0 { bail!("{} is not a feed", feed); }
            eprintln!("Changed {} backlog to {}", feed, n);
        }
    }

    Ok(())
}

