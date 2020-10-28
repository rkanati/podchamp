
use url::Url;

pub use clap::Clap;

#[derive(Clap)]
#[clap(about, author, version)]
pub struct Options {
    /// Path to Podchamp's database file
    #[clap(long, default_value = "", env = "PODCHAMP_DATABASE_PATH", parse(from_os_str))]
    pub database_path: std::ffi::OsString,

    /// Command to invoke when downloading episodes
    ///
    /// This command is invoked with the URI of the file to be downloaded as its lone argument.
    /// Various episode metadata is injected into its environment, with variables names starting
    /// with `PODCHAMP_`
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

#[derive(Clap)]
pub enum Command {
    /// Add a feed
    Add {
        /// A name for the feed
        name: String,

        /// The feed's RSS link
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

#[derive(Clap)]
pub enum Modification {
    /// Set the feed's RSS link
    Link {
        /// The new link
        link: Url,
    },

    /// Set the number of most-recent episodes to fetch
    Backlog {
        n: std::num::NonZeroU32,
    },
}

