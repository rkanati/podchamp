
#[macro_use]
extern crate diesel;

#[macro_use]
extern crate diesel_migrations;

use thiserror::Error;

embed_migrations!();

pub mod models;
pub mod schema;

pub fn run_migrations(db: &diesel::sqlite::SqliteConnection) -> anyhow::Result<()> {
    embedded_migrations::run(db)?;
    Ok(())
}

pub struct Database {
    conn: diesel::sqlite::SqliteConnection,
}

#[derive(Debug, Error)]
pub enum OpenDatabaseError {
    #[error("invalid database path")]
    InvalidPath,
    #[error("creating database directory")]
    CreateDirectory(std::io::Error),
    #[error("opening database")]
    Diesel(#[from] diesel::result::ConnectionError),
}

impl Database {
    pub fn open(path: &std::path::Path) -> Result<Database, OpenDatabaseError> {
        let dir = path.parent().ok_or(OpenDatabaseError::InvalidPath)?;
        std::fs::create_dir_all(dir).map_err(OpenDatabaseError::CreateDirectory)?;

        let path = path.to_str().ok_or(OpenDatabaseError::InvalidPath)?;
        use diesel::prelude::*;
        let conn = SqliteConnection::establish(path)?;

        let db = Database{conn};
        Ok(db)
    }
}

impl std::ops::Deref for Database {
    type Target = diesel::sqlite::SqliteConnection;

    fn deref(&self) -> &Self::Target {
        &self.conn
    }
}

