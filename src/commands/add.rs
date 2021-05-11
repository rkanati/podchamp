
use {
    crate::Anyhow,
    podchamp::Database,
    std::num::NonZeroU32,
    url::Url,
};

pub(crate)
async fn add(db: &mut Database, name: &str, link: &Url, backlog: Option<NonZeroU32>) -> Anyhow<()> {
    let backlog = backlog.or(NonZeroU32::new(1)).unwrap();
    db.add_feed(name, link, backlog)?;
    eprintln!("Added {}", name);
    Ok(())
}

