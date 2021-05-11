
use {
    crate::Anyhow,
    podchamp::Database,
};

pub(crate)
async fn rm(db: &mut Database, name: &str) -> Anyhow<()> {
    db.remove_feed(name)?;
    Ok(())
}

