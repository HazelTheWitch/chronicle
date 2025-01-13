pub mod bsky;
pub mod tumblr;

use std::{collections::HashMap, ops::Deref, sync::Arc};

use async_trait::async_trait;
use keyring::Entry;
use serde::{Deserialize, Serialize};
use sqlx::{types::chrono, Acquire, Sqlite, Transaction};
use url::Url;

use crate::{
    author::{self, AuthorQuery},
    models::{Author, Tag, Work},
    record::{Record, RecordDetails},
    Chronicle, ServiceError,
};

pub const SERVICE_NAME: &str = "chronicle";

#[derive(Deserialize, Serialize)]
struct StoredSecrets {
    secrets: HashMap<String, String>,
    previous: Option<HashMap<String, String>>,
}

#[async_trait]
pub trait Service {
    fn host_matches(&self, host: &str) -> bool;
    fn name(&self) -> &str;
    fn secrets(&self) -> &[&str];
    async fn authenticate(
        &self,
        secrets: &HashMap<String, String>,
        previous_result: Option<HashMap<String, String>>,
    ) -> Result<HashMap<String, String>, crate::Error>;
    async fn import(
        &self,
        chronicle: &crate::Chronicle,
        url: Url,
        records: &mut Vec<Record>,
        secrets: HashMap<String, String>,
        authentication: HashMap<String, String>,
    ) -> Result<(), crate::Error>;
}

lazy_static::lazy_static! {
    pub static ref SERVICES: Vec<Box<dyn Service + Send + Sync + 'static>> = vec![Box::new(bsky::Bsky::default()), Box::new(tumblr::Tumblr::default())];
}

pub fn write_secrets(
    service_name: &str,
    secrets: HashMap<String, String>,
) -> Result<(), crate::Error> {
    let user = whoami::username();

    let entry = Entry::new_with_target(service_name, SERVICE_NAME, &user).map_err(|error| {
        crate::Error::Keyring {
            service: service_name.to_owned(),
            error,
        }
    })?;

    entry
        .set_secret(&bincode::serialize(&StoredSecrets {
            secrets,
            previous: None,
        })?)
        .map_err(|error| crate::Error::Keyring {
            service: service_name.to_owned(),
            error,
        })?;

    Ok(())
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

        let Some(service) = SERVICES.iter().find(|s| s.host_matches(&host)) else {
            return Err(crate::Error::Generic(format!(
                "could not find service for {host}"
            )));
        };

        let user = whoami::username();

        let entry =
            Entry::new_with_target(service.name(), SERVICE_NAME, &user).map_err(|error| {
                crate::Error::Keyring {
                    service: service.name().to_owned(),
                    error,
                }
            })?;

        let mut secrets: StoredSecrets =
            bincode::deserialize(&entry.get_secret().map_err(|error| crate::Error::Keyring {
                service: service.name().to_owned(),
                error,
            })?)?;

        for secret in service.secrets() {
            if !secrets.secrets.contains_key(*secret) {
                return Err(crate::Error::Generic(format!(
                    "{} does not have secret: {secret}",
                    service.name()
                )));
            }
        }

        secrets.previous = Some(
            service
                .authenticate(&secrets.secrets, secrets.previous)
                .await?,
        );

        entry
            .set_secret(&bincode::serialize(&secrets)?)
            .map_err(|error| crate::Error::Keyring {
                service: service.name().to_owned(),
                error,
            })?;

        service
            .import(
                &chronicle,
                url.clone(),
                &mut records,
                secrets.secrets,
                secrets.previous.expect("just filled"),
            )
            .await?;

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
