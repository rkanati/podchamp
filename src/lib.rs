
#[macro_use]
extern crate diesel;

#[macro_use]
extern crate diesel_migrations;

embed_migrations!();

pub mod models;
pub mod schema;

pub fn run_migrations(db: &diesel::sqlite::SqliteConnection) -> anyhow::Result<()> {
    embedded_migrations::run(db)?;
    Ok(())
}

