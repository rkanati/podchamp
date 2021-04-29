
use crate::{Anyhow, models, schema, Db};

pub(crate)
async fn ls(db: &Db) -> Anyhow<()> {
    use{diesel::prelude::*, schema::feeds::dsl as dsl};
    let results = dsl::feeds.load::<models::Feed>(db)?;
    if results.is_empty() {
        eprintln!("No feeds. You can add one with `podchamp add`.");
    }
    else {
        for feed in results {
            // TODO make an effort to tabulate
            println!("{:16} {}", feed.name, feed.uri);
        }
    }

    Ok(())
}

