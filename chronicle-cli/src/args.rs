use std::path::PathBuf;

use chronicle::record::RecordDetails;
use clap::{Args, Parser, Subcommand};
use directories::ProjectDirs;
use lazy_static::lazy_static;
use tracing::Level;
use url::Url;

use crate::PROJECT_DIRS;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Arguments {
    #[arg(short, long, env = "CHRONICLE_LOG", default_value_t = Level::INFO)]
    pub log_level: Level,
    #[arg(long, default_value = PROJECT_DIRS.data_dir().join("chronicle.log").into_os_string())]
    pub log_location: PathBuf,
    #[arg(short, long, default_value = PROJECT_DIRS.config_dir().join("config.toml").into_os_string())]
    pub config: PathBuf,
    #[command(subcommand)]
    pub command: Option<Command>,
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
    Service {
        #[command(subcommand)]
        command: ServiceCommand,
    },
    WriteConfig,
    Tag {
        #[arg(short, long)]
        targets: Vec<String>,
        tags: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum ServiceCommand {
    Login { service: String },
    List,
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
    pub url: Option<Url>,
    /// The caption associated with the work.
    #[arg(long)]
    pub caption: Option<String>,
}

impl From<WorkDetails> for RecordDetails {
    fn from(
        WorkDetails {
            tags,
            title,
            author,
            url,
            caption,
        }: WorkDetails,
    ) -> Self {
        Self {
            tags,
            title,
            author,
            url,
            caption,
        }
    }
}
