
use {
    crate::{Anyhow, options::Modification},
    podchamp::Database,
};

pub(crate)
async fn mod_(db: &mut Database, feed: &str, how: &Modification) -> Anyhow<()> {
    match how {
        Modification::Link{link} => {
            db.set_link(feed, link)?;
            eprintln!("Changed {} feed link to {}", feed, link);
        }

        Modification::Backlog{n} => {
            db.set_backlog(feed, *n)?;
            eprintln!("Changed {} backlog to {}", feed, n);
        }
    }

    Ok(())
}

