
use url::Url;

#[derive(Clone, Debug)]
pub struct DatabasePath(std::path::PathBuf);

impl std::ops::Deref for DatabasePath {
    type Target = std::path::Path;
    fn deref(&self) -> &std::path::Path {
        &self.0
    }
}

impl Default for DatabasePath {
    fn default() -> Self {
        let dirs = directories::ProjectDirs::from("", "", "podchamp").unwrap();
        let path = dirs.data_dir().join("podchamp.sqlite");
        DatabasePath(path)
    }
}

impl std::str::FromStr for DatabasePath {
    type Err = <std::path::PathBuf as std::str::FromStr>::Err;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        std::str::FromStr::from_str(s).map(DatabasePath)
    }
}

impl std::fmt::Display for DatabasePath {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0.to_str().unwrap())
    }
}

#[derive(Clone, Debug)]
pub struct RuntimeDirPath(std::path::PathBuf);

impl std::ops::Deref for RuntimeDirPath {
    type Target = std::path::Path;
    fn deref(&self) -> &std::path::Path {
        &self.0
    }
}

impl Default for RuntimeDirPath {
    fn default() -> Self {
        let dirs = directories::ProjectDirs::from("", "", "podchamp").unwrap();
        let path = dirs.runtime_dir().unwrap();
        RuntimeDirPath(path.into())
    }
}

impl std::str::FromStr for RuntimeDirPath {
    type Err = <std::path::PathBuf as std::str::FromStr>::Err;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        std::str::FromStr::from_str(s).map(RuntimeDirPath)
    }
}

impl std::fmt::Display for RuntimeDirPath {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0.to_str().unwrap())
    }
}

#[derive(clap::Parser)]
#[clap(about, author, version)]
pub struct Options {
    /// Path to Podchamp's database file
    #[clap(long, default_value_t, env = "PODCHAMP_DATABASE_PATH")]
    pub database_path: DatabasePath,

    /// Path to a temporary folder
    #[clap(long, default_value_t, env = "PODCHAMP_RUNTIME_DIR")]
    pub runtime_dir_path: RuntimeDirPath,

    /// Command to invoke when downloading episodes
    ///
    /// This command is invoked with the URI of the file to be downloaded as its lone argument.
    /// Various feed and episode metadata is injected into its environment, in variables with names
    /// starting with `PODCHAMP_`
    #[clap(long, default_value = "wget", env = "PODCHAMP_DOWNLOADER")]
    pub downloader: String,

    /// The format for the episode's date passed to the downloader in `PODCHAMP_DATE`
    ///
    /// See `strftime(3)` for how to specify this
    #[clap(long, default_value = "%F", env = "PODCHAMP_DATE_FORMAT")]
    pub date_format: String,

    #[clap(subcommand)]
    pub command: Command
}

impl Options {
    pub fn load() -> Self {
        use clap::Parser;
        Self::parse()
    }
}

#[derive(clap::Parser)]
pub enum Command {
    /// Add a feed
    Add {
        /// A name for the feed
        name: String,

        /// The feed's link
        link: Url,

        /// Number of most-recent episodes to fetch. Defaults to 1.
        #[clap(short = 'n', long = "backlog")]
        backlog: Option<std::num::NonZeroU32>,
    },

    /// Remove a feed
    #[clap(alias = "remove")]
    Rm {
        /// The feed to remove
        name: String,
    },

    /// List feeds
    #[clap(alias = "list")]
    Ls,

    /// Modify a feed's settings
    #[clap(alias = "modify")]
    Mod {
        /// The name of the feed to modify
        feed: String,

        #[clap(subcommand)]
        how: Modification,
    },

    /// Fetch latest episodes
    Fetch {
        /// A particular feed to fetch
        feed: Option<String>,
    },

    /// Forget about episodes fetched previously
    Reset {
        /// The feed whose progress should be forgotten
        feed: String,
    },
}

#[derive(clap::Parser)]
pub enum Modification {
    /// Set the feed's link
    Link {
        /// The new link
        link: Url,
    },

    /// Set the number of most-recent episodes to fetch
    Backlog {
        n: std::num::NonZeroU32,
    },
}

