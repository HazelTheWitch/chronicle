use std::{fs::{self, OpenOptions}, path::PathBuf, sync::Condvar};

use anyhow::bail;
use chronicle::{import::{import, import_from_link}, record::Record, Arguments, Command, ServiceCredentials, WorkDetails, BSKY_EMAIL, BSKY_PASSWORD, CONFIG, PROJECT_DIRS, SERVICE_NAME};
use clap::Parser;
use sqlx::{migrate, SqlitePool};
use tracing::{info, level_filters::LevelFilter, warn, Level};
use tracing_subscriber::EnvFilter;
use url::Url;
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter(EnvFilter::builder().with_default_directive(LevelFilter::from_level(Level::INFO).into()).from_env()?).init();

    let args = Arguments::parse();

    if !fs::exists(&CONFIG.database_path)? {
        if let Some(directory) = CONFIG.database_path.parent() {
            fs::create_dir_all(directory)?;
        }

        OpenOptions::new().create(true).write(true).open(&CONFIG.database_path)?;
    }

    let database_url = format!("sqlite:///{path}", path = CONFIG.database_path.to_string_lossy());

    let db = SqlitePool::connect(&database_url).await?;

    migrate!().run(&db).await?;

    match args.command {
        Command::Search { query } => todo!(),
        Command::Import { url, details } => {
            import_from_link(&url).await?;
        },
        Command::Add {
            path,
            copy,
            details: WorkDetails {
                tags,
                title,
                author,
                url,
                caption,
            },
        } => {
            if !fs::metadata(&path)?.is_file() {
                bail!("Provided path is not a file.");
            }

            if let Some(url) = &url {
                if Url::parse(url).is_err() {
                    warn!("Provided url is invalid: {url}");
                }
            }

            let relative_path = if copy {
                match path.strip_prefix(&CONFIG.data_path) {
                    Ok(relative_path) => relative_path,
                    Err(_) => {
                        let file_name = if let Some(extension) = path.extension() {
                            format!("{}.{}", Uuid::new_v4(), extension.to_string_lossy())
                        } else {
                            Uuid::new_v4().to_string()
                        };

                        let new_path = CONFIG.data_path.join(&file_name);
                        
                        fs::copy(&path, &new_path)?;

                        info!("Copied file to DATA_DIR/{file_name}");

                        &PathBuf::from(file_name)
                    }
                }
            } else {
                path.strip_prefix(&CONFIG.data_path)?
            };

            import(&db, Record { path: relative_path.to_owned(), tags, title, url, author, caption }).await?;
        },
        Command::WriteConfig => {
            let config_path = PROJECT_DIRS.config_dir().join("config.toml");

            fs::write(config_path, toml::to_string_pretty(&*CONFIG)?)?;
        },
        Command::Login { service } => {
            match service {
                ServiceCredentials::Bsky { email, password } => {
                    let password_entry = keyring::Entry::new(SERVICE_NAME, BSKY_PASSWORD)?;
                    password_entry.set_password(&password)?;

                    let email_entry = keyring::Entry::new(SERVICE_NAME, BSKY_EMAIL)?;
                    email_entry.set_password(&email)?;
                },
            }
        },
    }

    Ok(())
}
