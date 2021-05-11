
use {
    crate::Anyhow,
    podchamp::{Database, GetFeeds},
};

pub(crate)
async fn ls(db: &Database) -> Anyhow<()> {
    let results = db.get_feeds(GetFeeds::All)?;
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

