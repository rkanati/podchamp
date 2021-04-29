
use crate::{Anyhow, schema, Db};

pub(crate)
async fn rm(db: &Db, name: &str) -> Anyhow<()> {
    use{diesel::prelude::*, schema::feeds::dsl as dsl};
    let n = diesel::delete(dsl::feeds.filter(dsl::name.eq(name)))
        .execute(db)?;
    if n == 0 {
        eprintln!("{} is not a feed", name);
    }

    Ok(())
}

