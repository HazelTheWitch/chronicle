pub mod author;
pub mod id;
pub mod import;
pub mod models;
pub mod record;
pub mod search;
pub mod tag;
pub mod utils;

use std::{fs, io, path::PathBuf};

use models::ModelKind;
use record::Record;
use search::SearchError;
use serde::{Deserialize, Serialize};
use sqlx::{migrate::MigrateError, SqlitePool};
use tracing::{debug, error, info};

pub const DEFAULT_CONFIG: &str = include_str!("../../default_config.toml");

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

pub struct Chronicle {
    pub pool: SqlitePool,
    pub config_path: PathBuf,
    pub config: Config,
}

impl Chronicle {
    pub async fn from_path(config_path: impl Into<PathBuf>) -> Result<Self, Error> {
        let path = config_path.into();

        info!("Loading config from {path:?}");

        let text = fs::read_to_string(&path)?;

        let mut config: Config = toml::from_str(&text)?;

        config.expand_paths()?;

        debug!("Loaded config: {config:?}");

        let database_url = format!(
            "sqlite:///{path}",
            path = config.database_path.to_string_lossy()
        );

        let pool = SqlitePool::connect(&database_url).await?;

        sqlx::migrate!().run(&pool).await?;

        Ok(Chronicle {
            pool,
            config_path: path,
            config,
        })
    }

    pub fn record_from_path(&self, path: impl Into<PathBuf>) -> Result<Record, Error> {
        let path = path.into();

        let hash = bytemuck::cast(crc32fast::hash(&fs::read(
            &self.config.data_path.join(&path),
        )?));

        Ok(Record {
            path,
            hash,
            details: Default::default(),
        })
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
    #[error("error communicating with service")]
    Http(#[from] reqwest::Error),
    #[error("invalid url")]
    Url(#[from] url::ParseError),
    #[error("{0}")]
    Generic(String),
    #[error("{kind} '{identifier}' is ambiguous")]
    Ambiguous { kind: ModelKind, identifier: String },
    #[error(transparent)]
    Search(#[from] SearchError),
    #[error(transparent)]
    Migration(#[from] MigrateError),
    #[error("could not expand {0}")]
    Expansion(String),
}

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("error communicating with bsky")]
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
