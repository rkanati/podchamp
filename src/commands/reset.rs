
use {
    crate::Anyhow,
    podchamp::Database,
};

pub(crate)
async fn reset(db: &mut Database, feed: Option<&str>) -> Anyhow<()> {
    if feed.is_none() {
        todo!("reset for all feeds");
    }

    db.reset_register(feed.unwrap())?;

    eprintln!("Progress reset for {}", feed.unwrap_or("all feeds"));
    Ok(())
}

