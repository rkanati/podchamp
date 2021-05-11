
#![forbid(unsafe_code)]
#![feature(try_blocks)]

mod fetch;
mod options;

use {
    crate::{fetch::fetch, options::*},
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
        Command::Add{name, link, backlog} => {
            let backlog = backlog.or(std::num::NonZeroU32::new(1)).unwrap();
            db.add_feed(name, link, backlog)?;
            eprintln!("Added {}", name);
        }

        Command::Rm{name} => {
            db.remove_feed(name)?;
        }

        Command::Ls => {
            let results = db.get_feeds(podchamp::GetFeeds::All)?;
            if results.is_empty() { eprintln!("No feeds. You can add one with `podchamp add`."); }
            for feed in results {
                // TODO tabulate
                println!("{:16} {}", feed.name, feed.uri);
            }
        }

        Command::Mod{feed, how} => {
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
        }

        Command::Reset{feed} => {
            db.reset_register(&feed)?;
            eprintln!("Progress reset for {}", feed);
        }

        Command::Fetch{feed} => fetch(&mut db, feed.as_deref(), &opts, now).await?,
    }

    instance.done();
    Ok(())
}

