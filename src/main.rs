
#![forbid(unsafe_code)]
#![feature(try_blocks)]

mod commands;
mod options;

use {
    crate::options::*,
    anyhow::anyhow,
    chrono::prelude::*,
};

pub(crate) use anyhow::Result as Anyhow;
pub(crate) use diesel::sqlite::SqliteConnection as Db;

fn open_db(path: &std::path::Path) -> Anyhow<Db> {
    let bad_path_error = || anyhow!("Invalid database path");

    let dir = path.parent().ok_or_else(bad_path_error)?;
    std::fs::create_dir_all(dir)?;

    let path = path.to_str().ok_or_else(bad_path_error)?;
    use diesel::prelude::*;
    let db = SqliteConnection::establish(path)?;

    Ok(db)
}

#[derive(Clone)]
struct SingleInstance(std::sync::Arc<std::sync::Mutex<Option<std::path::PathBuf>>>);

impl SingleInstance {
    fn new(rt_path: &std::path::Path) -> Anyhow<Self> {
        std::fs::create_dir_all(rt_path)?;

        let lockdir_path = rt_path.join("podchamp.lock.d");
        std::fs::create_dir(&lockdir_path)?;

        let pid = format!("{}", std::process::id());
        std::fs::write(lockdir_path.join("pid"), &pid)?;

        let utx = std::sync::Mutex::new(Some(lockdir_path));
        Ok(SingleInstance(std::sync::Arc::new(utx)))
    }

    fn done(&self) {
        if let Ok(mut lock) = self.0.lock() {
            if let Some(path) = lock.take() {
                let _ = std::fs::remove_dir_all(&path);
            }
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Anyhow<()> {
    let now = Utc::now();

    let opts = options::Options::load();

    let instance = SingleInstance::new(&opts.runtime_dir_path)?;
    std::panic::set_hook({
        let hook = std::panic::take_hook();
        let instance = instance.clone();
        Box::new(move |info| {
            instance.done();
            (hook)(info)
        })
    });

    let db = open_db(&opts.database_path)?;

    match &opts.command {
        Command::Add{name, link, backlog} => commands::add(&db, name, link, *backlog).await?,
        Command::Rm{name} => commands::rm(&db, name).await?,
        Command::Ls => commands::ls(&db).await?,
        Command::Mod{feed, how} => commands::mod_(&db, feed, how).await?,
        Command::Reset{feed} => commands::reset(&db, Some(feed)).await?,
        Command::Fetch{feed} => commands::fetch(now, &opts, &db, feed.as_deref()).await?,
    }

    instance.done();
    Ok(())
}

