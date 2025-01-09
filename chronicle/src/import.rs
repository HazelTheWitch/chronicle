pub mod bsky;

use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use tokio::sync::RwLock;
use url::Url;

use crate::{
    models::{AuthorId, Work},
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
    services: Vec<Box<dyn Service + Send + Sync + 'static>>,
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

pub async fn work_present_with_link(
    db: &sqlx::SqlitePool,
    link: &str,
) -> Result<bool, sqlx::Error> {
    Ok(sqlx::query(r#"SELECT 1 FROM works WHERE url = ? LIMIT 1;"#)
        .bind(link)
        .fetch_optional(db)
        .await?
        .is_some())
}

impl Work {
    pub async fn import_works_from_url(
        chronicle: &Chronicle,
        url: &str,
        provided_details: Option<RecordDetails>,
        author_id: Option<AuthorId>,
    ) -> Result<Vec<Work>, crate::Error> {
        let url = Url::parse(url)?;

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

        let tx = chronicle.pool.begin().await?;

        for record in records {
            works.push(Self::create_from_record(&chronicle, &record, author_id).await?);
        }

        tx.commit().await?;
        Ok(works)
    }

    pub async fn create_from_record(
        chronicle: &Chronicle,
        record: &Record,
        author_id: Option<AuthorId>,
    ) -> Result<Work, crate::Error> {
        let tx = chronicle.pool.begin().await?;

        let work: Work = sqlx::query_as("INSERT INTO works(path, url, author_id, title, caption, hash) VALUES (?, ?, ?, ?, ?, ?) RETURNING *;")
            .bind(&record.path.to_string_lossy())
            .bind(&record.details.url.as_ref().map(|url| url.to_string()))
            .bind(&author_id)
            .bind(&record.details.title)
            .bind(&record.details.caption)
            .bind(&record.hash)
            .fetch_one(&chronicle.pool)
            .await?;

        for tag in record.details.tags.iter() {
            work.tag(&chronicle, tag).await?;
        }

        tx.commit().await?;

        Ok(work)
    }
}
