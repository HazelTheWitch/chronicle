mod args;
mod logging;
mod ui;

use std::{
    fs::{self, OpenOptions},
    io::{stdin, stdout, Write},
    path::PathBuf,
};

use anyhow::bail;
use args::{Arguments, Command, ServiceCommand};
use chronicle::{
    author::AuthorQuery,
    import::SERVICES,
    models::{Author, Tag, Work},
    record::Record,
    search::QueryTerm,
    Chronicle, Config,
};
use clap::Parser;
use directories::ProjectDirs;
use lazy_static::lazy_static;
use logging::initialize_logging;
use tokio::sync::OnceCell;
use tracing::{error, info, warn};
use uuid::Uuid;

lazy_static! {
    pub static ref ARGUMENTS: Arguments = Arguments::parse();
    pub static ref PROJECT_DIRS: ProjectDirs =
        ProjectDirs::from("dev.setaria", "HazelTheWitch", "chronicle-cli")
            .expect("could not get project directories");
}

static CHRONICLE: OnceCell<Chronicle> = OnceCell::const_new();

pub async fn get_chronicle() -> &'static Chronicle {
    CHRONICLE
        .get_or_init(|| async {
            Chronicle::from_path(ARGUMENTS.config.clone())
                .await
                .expect("could not load config file")
        })
        .await
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if matches!(ARGUMENTS.command, Some(Command::WriteConfig)) {
        fs::write(&ARGUMENTS.config, chronicle::DEFAULT_CONFIG)?;
        return Ok(());
    }

    initialize_logging()?;

    let chronicle = get_chronicle().await;

    if !fs::exists(&chronicle.config.database_path)? {
        if let Some(directory) = chronicle.config.database_path.parent() {
            fs::create_dir_all(directory)?;
        }

        OpenOptions::new()
            .create(true)
            .write(true)
            .open(&chronicle.config.database_path)?;
    }

    if !fs::exists(&chronicle.config.data_path)? {
        fs::create_dir_all(&chronicle.config.data_path)?;
    }

    let tx = chronicle.pool.begin().await?;

    match &ARGUMENTS.command {
        Some(Command::List) => {
            let works = Work::get_all(&chronicle).await?;

            for work in works {
                println!("{} ({})", work.path, work.work_id);
                println!(
                    "Title: {}",
                    work.title.unwrap_or_else(|| String::from("NONE"))
                );
                println!(
                    "Caption: {}",
                    work.caption.unwrap_or_else(|| String::from("NONE"))
                );
                println!("Url: {}", work.url.unwrap_or_else(|| String::from("NONE")));
                println!();
            }
        }
        Some(Command::Search { destination, query }) => {
            let works = Work::search_by_str(&chronicle, &query).await?;

            println!("Found {} matches.", works.len());

            for work in &works {
                println!("{} {} {:?}", work.work_id, work.path, work.url,);
            }

            if !works.is_empty() {
                if let Some(destination) = destination {
                    fs::create_dir_all(&destination)?;

                    for work in &works {
                        fs::copy(
                            chronicle.config.data_path.join(&work.path),
                            destination.join(&work.path),
                        )?;
                    }
                }
            }
        }
        Some(Command::Import {
            link,
            force,
            details,
        }) => {
            if !force
                && Work::search(&chronicle, &QueryTerm::Url(link.clone()).into())
                    .await?
                    .len()
                    > 0
            {
                warn!("A work has already been chronicled with this url, if you want to repeat this operation pass --force");
                return Ok(());
            }

            Work::import_works_from_url(&chronicle, &link, Some(details.clone().into()), None)
                .await?;
        }
        Some(Command::Add {
            path,
            copy,
            details,
        }) => {
            if !fs::metadata(&path)?.is_file() {
                bail!("Provided path is not a file.");
            }

            let relative_path = if *copy {
                match path.strip_prefix(&chronicle.config.data_path) {
                    Ok(relative_path) => relative_path,
                    Err(_) => {
                        let file_name = if let Some(extension) = path.extension() {
                            format!("{}.{}", Uuid::new_v4(), extension.to_string_lossy())
                        } else {
                            Uuid::new_v4().to_string()
                        };

                        let new_path = chronicle.config.data_path.join(&file_name);

                        fs::copy(&path, &new_path)?;

                        info!("Copied file to DATA_DIR/{file_name}");

                        &PathBuf::from(file_name)
                    }
                }
            } else {
                path.strip_prefix(&chronicle.config.data_path)?
            };

            let author_id = match &details.author {
                Some(name) => {
                    let mut authors =
                        Author::get(&chronicle, AuthorQuery::Name(name.to_string())).await?;

                    match authors.len() {
                        0 => None,
                        1 => Some(authors.remove(0).author_id),
                        _ => anyhow::bail!("Author {name} is ambiguous"),
                    }
                }
                None => None,
            };

            let record =
                Record::from_path(&chronicle, relative_path.to_owned(), details.clone().into())?;

            Work::create_from_record(&chronicle, &record, author_id).await?;
        }
        Some(Command::Service {
            command: ServiceCommand::List,
        }) => {
            for service in SERVICES.services.iter() {
                println!("{}", service.name());
            }
        }
        Some(Command::Service {
            command: ServiceCommand::Login {
                service: service_name,
            },
        }) => {
            let service = SERVICES.services.iter().find(|s| s.name() == service_name);

            if let Some(service) = service {
                let mut buffer = String::new();
                let stdin = stdin();
                let mut stdout = stdout();

                for secret_key in service.secrets() {
                    write!(stdout, "{secret_key}: ")?;
                    stdout.flush()?;
                    stdin.read_line(&mut buffer)?;

                    if let Err(err) = service.write_secret(secret_key, buffer.trim()) {
                        error!("Could not write secret: {secret_key}: {err}");
                    }
                    buffer.clear();
                }
            } else {
                error!("Invalid service: {service_name}");
            }
        }
        Some(Command::Tag { targets, tags }) => {
            for target in targets {
                let target_tag = Tag::create(&chronicle, &target).await?;

                for tag in tags {
                    target_tag.tag(&chronicle, tag).await?;
                }
            }
        }
        _ => unreachable!(),
    }

    tx.commit().await?;

    Ok(())
}
