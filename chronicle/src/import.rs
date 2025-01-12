pub mod bsky;

use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use sqlx::{types::chrono, Acquire, Sqlite, Transaction};
use tokio::sync::RwLock;
use tracing::info;
use url::Url;

use crate::{
    author::{self, AuthorQuery},
    models::{Author, Tag, Work},
    record::{Record, RecordDetails},
    Chronicle,
};

pub const SERVICE_NAME: &str = "chronicle";

#[async_trait]
pub trait Service {
    fn host(&self) -> &str;
    async fn import(
        &self,
        chronicle: &crate::Chronicle,
        url: Url,
        records: &mut Vec<Record>,
        secrets: Arc<RwLock<HashMap<String, String>>>,
    ) -> Result<(), crate::Error>;
    fn secrets(&self) -> &[&str];

    fn write_secret(&self, key: &str, secret: &str) -> Result<(), keyring::Error> {
        let entry = keyring::Entry::new(SERVICE_NAME, key)?;

        entry.set_password(secret)?;

        Ok(())
    }

    fn has_secrets(&self, secrets: &HashMap<String, String>) -> bool {
        for secret in self.secrets() {
            if !secrets.contains_key(*secret) {
                return false;
            }
        }

        true
    }

    fn name(&self) -> &str {
        self.host()
    }
}

pub struct Services {
    pub services: Vec<Box<dyn Service + Send + Sync + 'static>>,
    secrets: Arc<RwLock<HashMap<String, String>>>,
}

lazy_static::lazy_static! {
    pub static ref SERVICES: Services = get_services();
}

fn get_services() -> Services {
    let services: Vec<Box<dyn Service + Send + Sync + 'static>> = vec![Box::new(bsky::Bsky)];

    Services {
        services,
        secrets: Default::default(),
    }
}

impl Work {
    pub async fn import_works_from_url(
        chronicle: &Chronicle,
        tx: &mut Transaction<'_, Sqlite>,
        url: &Url,
        provided_details: Option<&RecordDetails>,
    ) -> Result<Vec<Work>, crate::Error> {
        let Some(host) = url.host_str() else {
            return Err(crate::Error::Generic(String::from(
                "url does not have a host",
            )));
        };

        let mut records = Vec::with_capacity(6);

        for service in SERVICES.services.iter() {
            if host == service.host() {
                {
                    let mut secrets = SERVICES.secrets.write().await;

                    for secret_key in service.secrets() {
                        let secret_value = keyring::Entry::new(SERVICE_NAME, &secret_key)
                            .map_err(|error| crate::Error::Keyring {
                                service: service.name().to_owned(),
                                error,
                            })?
                            .get_password()
                            .map_err(|error| crate::Error::Keyring {
                                service: service.name().to_owned(),
                                error,
                            })?;
                        secrets.insert(secret_key.to_string(), secret_value.to_owned());
                    }
                }

                service
                    .import(
                        chronicle,
                        url.clone(),
                        &mut records,
                        SERVICES.secrets.clone(),
                    )
                    .await?;

                break;
            }
        }

        if let Some(provided_details) = provided_details {
            for record in records.iter_mut() {
                record.details.update(provided_details.clone());
            }
        }

        let mut works = Vec::with_capacity(records.len());

        let mut tx = tx.begin().await?;

        for record in records {
            works.push(Self::create_from_record(&mut tx, &record).await?);
        }

        tx.commit().await?;

        Ok(works)
    }

    pub async fn create_from_record(
        tx: &mut Transaction<'_, Sqlite>,
        record: &Record,
    ) -> Result<Work, crate::Error> {
        let author_id = if let Some(author_query) = &record.details.author {
            let mut authors = Author::get(tx, author_query).await?;

            if authors.len() == 1 {
                Some(authors.remove(0).author_id)
            } else if authors.is_empty() {
                if let AuthorQuery::Name(name) = author_query {
                    let author = Author::create(tx, name).await?;

                    Some(author.author_id)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        let work: Work = sqlx::query_as("INSERT INTO works(path, url, author_id, title, caption, hash, size) VALUES (?, ?, ?, ?, ?, ?, ?) RETURNING *;")
            .bind(&record.path.to_string_lossy())
            .bind(&record.details.url.as_ref().map(|url| url.to_string()))
            .bind(&author_id)
            .bind(&record.details.title)
            .bind(&record.details.caption)
            .bind(&record.hash)
            .bind(record.size as u32)
            .fetch_one(&mut **tx)
            .await?;

        if let Some(author_id) = author_id {
            if let Some(author_url) = &record.details.author_url {
                let author = Author::get_by_id(tx, &author_id)
                    .await?
                    .expect("author id exists but not found");

                author.add_url(tx, author_url).await?;
            }
        }

        for tag in record.details.tags.iter() {
            let tag = Tag::get_discriminated_or_create(tx, &tag.name, tag.discriminator.as_deref())
                .await?;
            work.tag(tx, &tag).await?;
        }

        Ok(work)
    }
}
