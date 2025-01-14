pub mod author;
pub mod http;
pub mod id;
pub mod import;
pub mod models;
pub(crate) mod parse;
pub mod record;
pub mod search;
pub mod tag;
pub mod utils;

use std::{
    fs::{self, create_dir_all},
    io,
    path::PathBuf,
};

use directories::ProjectDirs;
use http::start_http_server;
use models::ModelKind;
use parse::ParseError;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sqlx::{migrate::MigrateError, SqlitePool, Transaction};
use tokio::task::JoinHandle;
use tracing::{debug, error, info};

lazy_static::lazy_static! {
    pub static ref HTTP_CLIENT: Client = Client::new();
    pub static ref DEFAULT_CONFIG: String = toml::to_string_pretty(&Config::default()).unwrap();
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub database_path: PathBuf,
    pub data_path: PathBuf,
}

impl Config {
    pub fn expand_paths(&mut self) -> Result<(), Error> {
        self.data_path = PathBuf::from(
            shellexpand::full(&self.data_path.to_string_lossy())
                .map_err(|_| Error::Expansion(self.data_path.to_string_lossy().to_string()))?
                .to_string(),
        );
        self.database_path = PathBuf::from(
            shellexpand::full(&self.database_path.to_string_lossy())
                .map_err(|_| Error::Expansion(self.database_path.to_string_lossy().to_string()))?
                .to_string(),
        );

        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        let project_dirs = ProjectDirs::from("dev.setaria", "HazelTheWitch", "chronicle")
            .expect("could not get project directories");

        Self {
            database_path: project_dirs.data_dir().join("database.db"),
            data_path: project_dirs.data_dir().join("works"),
        }
    }
}

pub struct Chronicle {
    pub pool: SqlitePool,
    pub config: Config,
    pub http_task: JoinHandle<()>,
}

impl Chronicle {
    pub async fn begin(&self) -> Result<Transaction<'_, sqlx::Sqlite>, sqlx::Error> {
        self.pool.begin().await
    }

    pub async fn from_config(mut config: Config) -> Result<Self, Error> {
        config.expand_paths()?;

        if !fs::exists(&config.database_path)? {
            fs::create_dir_all(
                &config
                    .database_path
                    .parent()
                    .expect("could not create database directory"),
            )?;

            fs::OpenOptions::new()
                .write(true)
                .create(true)
                .open(&config.database_path)?;
        }

        if !fs::exists(&config.data_path)? {
            create_dir_all(&config.data_path)?;
        }

        debug!("Loaded config: {config:?}");

        let database_url = format!(
            "sqlite:///{path}",
            path = config.database_path.to_string_lossy()
        );

        let pool = SqlitePool::connect(&database_url).await?;

        sqlx::migrate!().run(&pool).await?;

        let http_task = tokio::spawn(start_http_server());

        Ok(Chronicle {
            pool,
            config,
            http_task,
        })
    }

    pub async fn from_path(config_path: impl Into<PathBuf>) -> Result<Self, Error> {
        let path = config_path.into();

        info!("Loading config from {path:?}");

        let text = fs::read_to_string(&path)?;

        let config: Config = toml::from_str(&text)?;

        Self::from_config(config).await
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Sql(#[from] sqlx::Error),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("could not deserialize config file")]
    Toml(#[from] toml::de::Error),
    #[error("invalid url for service {service}: {url}")]
    InvalidUrl {
        service: &'static str,
        url: url::Url,
    },
    #[error(transparent)]
    Service(#[from] ServiceError),
    #[error("could not get secrets for service {service}: {error}")]
    Keyring {
        service: String,
        error: keyring::Error,
    },
    #[error("error communicating with service: {0}")]
    Http(#[from] reqwest::Error),
    #[error("invalid url")]
    Url(#[from] url::ParseError),
    #[error("{0}")]
    Generic(String),
    #[error("{kind} '{identifier}' is ambiguous")]
    Ambiguous { kind: ModelKind, identifier: String },
    #[error(transparent)]
    Search(#[from] ParseError),
    #[error(transparent)]
    Migration(#[from] MigrateError),
    #[error("could not expand {0}")]
    Expansion(String),
    #[error("{kind} not found")]
    NotFound { kind: ModelKind },
    #[error("could not deserialize secrets")]
    Secret(#[from] bincode::Error),
    #[error("oauth2 error {0}")]
    Oauth2(Box<dyn std::error::Error + Send + Sync>),
}

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("error communicating with bsky: {0}")]
    Bsky(Box<dyn std::error::Error + Send + Sync>),
}

impl<E> From<atrium_api::xrpc::Error<E>> for ServiceError
where
    E: std::fmt::Debug + std::fmt::Display + Send + Sync + 'static,
{
    fn from(value: atrium_api::xrpc::Error<E>) -> Self {
        Self::Bsky(Box::new(value))
    }
}

impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
