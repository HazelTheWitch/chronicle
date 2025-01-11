use std::{fmt::Display, path::PathBuf};

use chronicle::{author::AuthorQuery, record::RecordDetails, search::Query, tag::TagExpression};
use clap::{Args, Parser, Subcommand, ValueEnum};
use tracing::Level;
use url::Url;

use crate::PROJECT_DIRS;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Arguments {
    #[arg(short, long, env = "CHRONICLE_LOG", default_value_t = Level::INFO)]
    pub log_level: Level,
    #[arg(long, default_value = PROJECT_DIRS.data_dir().join("chronicle.log").into_os_string())]
    pub log_location: PathBuf,
    #[arg(short, long, default_value = PROJECT_DIRS.config_dir().join("config.toml").into_os_string())]
    pub config: PathBuf,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Operations with works
    Work {
        #[command(subcommand)]
        command: WorkCommand,
    },
    /// Operations with tags
    Tag {
        /// The tagging expression to execute
        ///
        /// This should take the form of
        ///
        /// [<search query>/]tag1/(tag2,tag3)/tag4
        expression: TagExpression,
    },
    /// Operations with authors
    #[command(alias = "artist")]
    Author {
        #[command(subcommand)]
        command: AuthorCommand,
    },
    /// List and authenticate importers
    Service {
        #[command(subcommand)]
        command: ServiceCommand,
    },
    /// Bulk operations
    Bulk {
        /// The number of tasks to use when importing
        #[arg(short, long, default_value_t = 4)]
        tasks: usize,
        #[command(subcommand)]
        command: BulkCommand,
    },
}

#[derive(Debug, Subcommand)]
pub enum BulkCommand {
    /// Import a list of urls
    Import {
        /// The path to a file containing a list of urls
        path: PathBuf,
        #[command(flatten)]
        details: BulkWorkDetails,
    },
    /// Import a list of paths as works
    Add {
        /// The path to a file containing a list of paths
        path: PathBuf,
        #[command(flatten)]
        details: BulkWorkDetails,
    },
    /// Performs a series of tag operations
    Tag {
        /// The path to a file containing a list of tag operations
        ///
        /// The format of each line of this file should be the same as the input for `chronicle tag`
        path: PathBuf,
    },
}

#[derive(Debug, Subcommand)]
pub enum WorkCommand {
    /// Chronicle a local file as a work
    Add {
        /// The local path to the file
        path: PathBuf,
        #[command(flatten)]
        details: WorkDetails,
    },
    /// Chronicle a link as work(s)
    Import {
        /// The source of the work(s)
        source: Url,
        #[command(flatten)]
        details: WorkDetails,
    },
    /// Search and display works
    Search {
        #[command(flatten)]
        display_options: WorkDisplayOptions,
        /// The query to search for, if omitted the query is read from stdin
        query: Option<Query>,
    },
    /// List all works chronicled
    List {
        #[command(flatten)]
        display_options: WorkDisplayOptions,
    },
}

#[derive(Debug, Subcommand)]
pub enum AuthorCommand {
    /// List all authors
    List {
        #[command(flatten)]
        display_options: AuthorDisplayOptions,
    },
    /// Assign an alias to an author
    Alias {
        /// The author to alias
        ///
        /// Can be a name, id, or url
        query: AuthorQuery,
        /// The alias to assgn to the author
        alias: String,
    },
    /// Assign a url to an author
    AddUrl {
        /// The author to assign the url to
        ///
        /// Can be a name, id, or url
        query: AuthorQuery,
        /// The url to assign to the author
        url: Url,
    },
}

#[derive(Debug, Subcommand)]
pub enum ServiceCommand {
    /// Login to a service
    Login {
        /// The service to log into
        service: Option<String>,
    },
    /// List all services available
    List,
}

#[derive(Debug, Args)]
pub struct WorkDisplayOptions {
    /// Specifies which columns to display
    #[arg(short, long, value_enum, default_values_t = vec![WorkColumn::Id, WorkColumn::Size, WorkColumn::Url])]
    pub columns: Vec<WorkColumn>,
}

#[derive(Debug, Args)]
pub struct AuthorDisplayOptions {
    /// Specifies which columns to display
    #[arg(short, long, value_enum, default_values_t = vec![AuthorColumn::Id, AuthorColumn::Aliases, AuthorColumn::Urls])]
    pub columns: Vec<AuthorColumn>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum AuthorColumn {
    Id,
    Aliases,
    Urls,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum WorkColumn {
    Id,
    Path,
    Hash,
    Title,
    AuthorId,
    Caption,
    Url,
    Size,
}

impl Display for WorkColumn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkColumn::Id => write!(f, "ID"),
            WorkColumn::Path => write!(f, "PATH"),
            WorkColumn::Hash => write!(f, "HASH"),
            WorkColumn::Title => write!(f, "TITLE"),
            WorkColumn::AuthorId => write!(f, "AUTHOR_ID"),
            WorkColumn::Caption => write!(f, "CAPTION"),
            WorkColumn::Url => write!(f, "URL"),
            WorkColumn::Size => write!(f, "SIZE"),
        }
    }
}

impl Display for AuthorColumn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthorColumn::Id => write!(f, "ID"),
            AuthorColumn::Aliases => write!(f, "ALIASES"),
            AuthorColumn::Urls => write!(f, "URLS"),
        }
    }
}

#[derive(Debug, Default, Clone, Args)]
pub struct WorkDetails {
    /// A list of tags to associate with the work
    pub tags: Vec<String>,
    /// The title of the work
    #[arg(short, long)]
    pub title: Option<String>,
    /// The original author of the work
    #[arg(short, long)]
    pub author: Option<AuthorQuery>,
    /// The url to associate with the work
    #[arg(short, long)]
    pub url: Option<Url>,
    /// The caption associated with the work
    #[arg(short, long)]
    pub caption: Option<String>,
}

#[derive(Debug, Default, Clone, Args)]
pub struct BulkWorkDetails {
    /// A list of tags to associate with the works
    pub tags: Vec<String>,
    /// The title of the works
    #[arg(short, long)]
    pub title: Option<String>,
    /// The original author of the works
    #[arg(short, long)]
    pub author: Option<AuthorQuery>,
    /// The caption associated with the works
    #[arg(short, long)]
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
            author_url: None,
        }
    }
}

impl From<BulkWorkDetails> for RecordDetails {
    fn from(
        BulkWorkDetails {
            tags,
            title,
            author,
            caption,
        }: BulkWorkDetails,
    ) -> Self {
        Self {
            title,
            url: None,
            author,
            caption,
            tags,
            author_url: None,
        }
    }
}
