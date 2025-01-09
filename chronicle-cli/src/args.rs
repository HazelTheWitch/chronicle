#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Arguments {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    Search {
        #[arg(short, long)]
        destination: Option<PathBuf>,
        query: String,
    },
    List,
    Add {
        /// The path of the work.
        path: PathBuf,
        /// Copy the work to the data directory if not already there.
        #[arg(short, long)]
        copy: bool,
        #[command(flatten)]
        details: WorkDetails,
    },
    Import {
        /// The link to the work.
        link: String,
        /// Skip duplicate check.
        #[arg(short, long)]
        force: bool,
        #[command(flatten)]
        details: WorkDetails,
    },
    Login {
        #[command(subcommand)]
        service: ServiceCredentials,
    },
    WriteConfig,
    Tag {
        #[arg(short, long)]
        targets: Vec<String>,
        tags: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum ServiceCredentials {
    Bsky {
        identifier: String,
        password: String,
    },
}

#[derive(Default, Clone, Args)]
pub struct WorkDetails {
    /// A list of tags to associate with the work.
    pub tags: Vec<String>,
    /// The title of the work.
    #[arg(short, long)]
    pub title: Option<String>,
    /// The original author of the work.
    #[arg(short, long)]
    pub author: Option<String>,
    /// The url to associate with the work.
    #[arg(short, long)]
    pub url: Option<String>,
    /// The caption associated with the work.
    #[arg(long)]
    pub caption: Option<String>,
}
