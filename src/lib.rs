
#[macro_use]
extern crate diesel;

#[macro_use]
extern crate diesel_migrations;

use {
    chrono::{DateTime, Utc, NaiveDateTime},
    thiserror::Error,
    url::Url,
};

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
    #[error(transparent)]
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

// TODO remove/replace with explicit method once standard ops are added to `Database`
impl std::ops::Deref for Database {
    type Target = diesel::sqlite::SqliteConnection;

    fn deref(&self) -> &Self::Target {
        &self.conn
    }
}

#[derive(Debug, Error)]
pub enum AddFeedError {
    #[error("feed named {0} already in database")]
    NameTaken(String),
    #[error(transparent)]
    Database(#[from] diesel::result::Error),
}

impl Database {
    pub fn add_feed(&mut self,
        name: &str,
        link: &Url,
        backlog: std::num::NonZeroU32,
    ) -> Result<(), AddFeedError> {
        let feed = models::NewFeed {
            name,
            uri: link.as_str(),
            backlog: backlog.get() as i32,
            fetch_since: None
        };

        use diesel::{prelude::*, result::{Error, DatabaseErrorKind}};
        diesel::insert_into(schema::feeds::table)
            .values(&feed)
            .execute(&self.conn)
            .map_err(|e| match e {
                Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _)
                    => AddFeedError::NameTaken(name.to_owned()),
                e   => e.into(),
            })?;

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum RemoveFeedError {
    #[error("no feed named {0}")]
    NoSuchFeed(String),
    #[error(transparent)]
    Database(#[from] diesel::result::Error),
}


impl Database {
    pub fn remove_feed(&mut self, name: &str) -> Result<(), RemoveFeedError> {
        use{diesel::prelude::*, schema::feeds::dsl as dsl};
        let n = diesel::delete(dsl::feeds.filter(dsl::name.eq(name)))
            .execute(&self.conn)?;
        if n == 0 {
            return Err(RemoveFeedError::NoSuchFeed(name.into()));
        }

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum GetFeedsError {
    #[error(transparent)]
    Database(#[from] diesel::result::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GetFeeds<'n> {
    All,
    One(&'n str),
}

impl Database {
    pub fn get_feeds(&self, which: GetFeeds<'_>) -> Result<Vec<models::Feed>, GetFeedsError> {
        use{diesel::prelude::*, schema::feeds::dsl as feeds};

        let query = match which {
            GetFeeds::All       => feeds::feeds.into_boxed(),
            GetFeeds::One(name) => feeds::feeds.filter(feeds::name.eq(name)).into_boxed()
        };

        query.load::<models::Feed>(&self.conn)
            .map_err(GetFeedsError::Database)
    }
}

#[derive(Debug, Error)]
pub enum SetColumnError {
    #[error("no feed named {0}")]
    NoSuchFeed(String),
    #[error(transparent)]
    Database(#[from] diesel::result::Error),
}

impl Database {
    pub fn set_link(&mut self, feed: &str, link: &Url) -> Result<(), SetColumnError> {
        use{diesel::prelude::*, schema::feeds::dsl as dsl};
        let n = diesel::update(dsl::feeds.filter(dsl::name.eq(feed)))
            .set(dsl::uri.eq(link.as_str()))
            .execute(&self.conn)?;
        if n == 0 {
            return Err(SetColumnError::NoSuchFeed(feed.into()));
        }

        Ok(())
    }

    pub fn set_backlog(&mut self, feed: &str, backlog: std::num::NonZeroU32)
        -> Result<(), SetColumnError>
    {
        use{diesel::prelude::*, schema::feeds::dsl as dsl};
        let n = diesel::update(dsl::feeds.filter(dsl::name.eq(feed)))
            .set(dsl::backlog.eq(backlog.get() as i32))
            .execute(&self.conn)?;
        if n == 0 {
            return Err(SetColumnError::NoSuchFeed(feed.into()));
        }

        Ok(())
    }

    pub fn set_fetch_since(&mut self, feed: &str, since: &DateTime<Utc>)
        -> Result<(), SetColumnError>
    {
        use{diesel::prelude::*, schema::feeds::dsl as dsl};
        let n = diesel::update(dsl::feeds.filter(dsl::name.eq(feed)))
            .set(dsl::fetch_since.eq(since.naive_utc()))
            .execute(&self.conn)?;
        if n == 0 {
            return Err(SetColumnError::NoSuchFeed(feed.into()));
        }

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum ResetRegisterError {
    #[error(transparent)]
    Database(#[from] diesel::result::Error),
}

impl Database {
    pub fn reset_register(&mut self, feed: &str) -> Result<(), ResetRegisterError> {
        use diesel::prelude::*;

        use schema::{register::dsl as register, feeds::dsl as feeds};

        diesel::delete(register::register.filter(register::feed.eq(feed)))
            .execute(&self.conn)?;

        diesel::update(feeds::feeds.filter(feeds::name.eq(feed)))
            .set(feeds::fetch_since.eq::<Option<NaiveDateTime>>(None))
            .execute(&self.conn)?;

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum IsEpisodeRegisteredError {
    #[error(transparent)]
    Database(#[from] diesel::result::Error),
}

impl Database {
    pub fn is_episode_registered(&self, feed: &str, guid: &str)
        -> Result<bool, IsEpisodeRegisteredError>
    {
        use {diesel::prelude::*, schema::register::dsl as register};
        let n: i64 = register::register
            .filter(register::feed.eq(feed))
            .filter(register::guid.eq(guid))
            .count()
            .get_result(&self.conn)?;
        Ok(n != 0)
    }
}

#[derive(Debug, Error)]
pub enum RegisterEpisodeError {
    #[error(transparent)]
    Database(#[from] diesel::result::Error),
}

impl Database {
    pub fn register_episode(&mut self, feed: &str, guid: &str)
        -> Result<(), RegisterEpisodeError>
    {
        let registration = models::NewRegistration{feed, guid};
        use diesel::prelude::*;
        diesel::insert_into(schema::register::table)
            .values(&registration)
            .execute(&self.conn)?;
        Ok(())
    }
}

