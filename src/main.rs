
#![forbid(unsafe_code)]
#![feature(try_blocks)]

mod commands;
mod options;

use {
    crate::options::*,
    chrono::prelude::*,
};

pub(crate) use anyhow::Result as Anyhow;

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

    let mut db = podchamp::Database::open(&opts.database_path)?;

    match &opts.command {
        Command::Add{name, link, backlog} => commands::add(&mut db, name, link, *backlog).await?,
        Command::Rm{name} => commands::rm(&mut db, name).await?,
        Command::Ls => commands::ls(&db).await?,
        Command::Mod{feed, how} => commands::mod_(&mut db, feed, how).await?,
        Command::Reset{feed} => commands::reset(&mut db, Some(feed)).await?,
        Command::Fetch{feed} => commands::fetch(&mut db, feed.as_deref(), &opts, now).await?,
    }

    instance.done();
    Ok(())
}

