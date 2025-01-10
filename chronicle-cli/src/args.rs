use std::path::PathBuf;

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
        #[command(subcommand)]
        command: TagCommand,
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
pub enum TagCommand {
    /// Tag a set of works with tags
    Work {
        /// The search query to tag
        query: String,
        /// The tags to apply to each work in the query
        tags: Vec<String>,
    },
    /// Tag a set of tags
    Meta {
        /// The expression to perform tagging with, if omitted the expression is read from stdin
        ///
        /// Examples:
        /// - subtag/supertag
        /// - subtag/(sibling,tags)/supertag
        expression: Option<TagExpression>,
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
    /// Assig a url to an author
    AddUrl {
        /// The author to assign the url to
        ///
        /// Can be a name, id, or url
        query: AuthorQuery,
    },
}

#[derive(Debug, Subcommand)]
pub enum ServiceCommand {
    /// Login to a service
    Login {
        /// The service to log into, if omitted will prompt for each service in sequence
        service: Option<String>,
    },
    /// List all services available
    List,
}

#[derive(Debug, Args)]
pub struct WorkDisplayOptions {
    /// Specifies which columns to display
    #[arg(short, long, value_enum, default_values_t = vec![WorkColumn::Id, WorkColumn::Path])]
    pub columns: Vec<WorkColumn>,
}

#[derive(Debug, Args)]
pub struct AuthorDisplayOptions {
    /// Specifies which columns to display
    #[arg(short, long, value_enum, default_values_t = vec![AuthorColumn::Id, AuthorColumn::Aliases])]
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
}

#[derive(Debug, Default, Clone, Args)]
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
