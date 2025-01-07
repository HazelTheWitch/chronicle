pub mod import;
pub mod record;
pub mod tag;

use std::{fs, path::PathBuf};

use clap::{Args, Parser, Subcommand};
use directories::ProjectDirs;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use tracing::error;

lazy_static! {
    pub static ref PROJECT_DIRS: ProjectDirs =
        ProjectDirs::from("dev", "setaria", "chronicle").unwrap();
    pub static ref CONFIG: Config = get_config();
}

pub const SERVICE_NAME: &str = "chronicle";

pub const BSKY_EMAIL: &str = "bsky-email";
pub const BSKY_PASSWORD: &str = "bsky-password";

pub mod prelude {
    pub use crate::import::{import, import_from_link};
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Arguments {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    Search {
        query: String,
    },
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
        /// The url of the work.
        url: String,
        #[command(flatten)]
        details: WorkDetails,
    },
    Login {
        #[command(subcommand)]
        service: ServiceCredentials,
    },
    WriteConfig,
}

#[derive(Subcommand)]
pub enum ServiceCredentials {
    Bsky { email: String, password: String },
}

#[derive(Args)]
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

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub database_path: PathBuf,
    pub data_path: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            database_path: PROJECT_DIRS.data_dir().join("database.db"),
            data_path: PROJECT_DIRS.data_dir().join("works"),
        }
    }
}

pub fn get_config() -> Config {
    let config_path = PROJECT_DIRS.config_dir().join("config.toml");

    let Ok(config_data) = fs::read_to_string(&config_path) else {
        error!("Could not load config file.");

        return Config::default();
    };

    let Ok(config) = toml::from_str(&config_data) else {
        error!("Could not deserialize config file.");

        return Config::default();
    };

    config
}
