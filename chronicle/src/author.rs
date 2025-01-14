use std::{convert::Infallible, str::FromStr};

use sqlx::{Sqlite, Transaction};
use url::Url;

use crate::{
    models::{Author, AuthorId, AuthorName, AuthorUrl},
    Chronicle,
};

#[derive(Debug, Clone)]
pub enum AuthorQuery {
    Name(String),
    Id(AuthorId),
    Url(url::Url),
}

impl FromStr for AuthorQuery {
    type Err = Infallible;

    fn from_str(query: &str) -> Result<Self, Self::Err> {
        if let Ok(id) = i64::from_str(query) {
            return Ok(Self::Id(AuthorId(id)));
        } else if let Ok(url) = url::Url::from_str(query) {
            return Ok(Self::Url(url));
        }

        Ok(Self::Name(query.to_owned()))
    }
}

impl Author {
    pub async fn create(
        tx: &mut Transaction<'_, Sqlite>,
        name: &str,
    ) -> Result<Author, crate::Error> {
        let author: Author = sqlx::query_as("INSERT INTO authors DEFAULT VALUES RETURNING *;")
            .fetch_one(&mut **tx)
            .await?;

        sqlx::query("INSERT INTO author_names(author_id, name) VALUES (?, ?);")
            .bind(author.author_id)
            .bind(name)
            .execute(&mut **tx)
            .await?;

        Ok(author)
    }

    pub async fn get_by_id(
        tx: &mut Transaction<'_, Sqlite>,
        author_id: &AuthorId,
    ) -> Result<Option<Author>, crate::Error> {
        Ok(sqlx::query_as("SELECT * FROM authors WHERE author_id = ?;")
            .bind(&author_id)
            .fetch_optional(&mut **tx)
            .await?)
    }

    pub async fn get(
        tx: &mut Transaction<'_, Sqlite>,
        query: &AuthorQuery,
    ) -> Result<Vec<Author>, crate::Error> {
        Ok(match query {
            AuthorQuery::Name(name) => {
                sqlx::query_as("SELECT * FROM authors JOIN author_names ON authors.author_id = author_names.author_id WHERE name = ?;")
                    .bind(name)
                    .fetch_all(&mut **tx)
                    .await?
            },
            AuthorQuery::Id(id) => {
                sqlx::query_as("SELECT * FROM authors WHERE author_id = ?;")
                    .bind(id)
                    .fetch_all(&mut **tx)
                    .await?
            },
            AuthorQuery::Url(url) => {
                sqlx::query_as("SELECT * FROM authors JOIN author_urls ON authors.author_id = author_urls.author_id WHERE author_urls.url = ?;")
                    .bind(url.to_string())
                    .fetch_all(&mut **tx)
                    .await?
            },

        })
    }

    pub async fn get_author_urls(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
    ) -> Result<Vec<AuthorUrl>, crate::Error> {
        Ok(
            sqlx::query_as("SELECT * FROM author_urls WHERE author_id = ?;")
                .bind(&self.author_id)
                .fetch_all(&mut **tx)
                .await?,
        )
    }

    pub async fn add_url(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        url: &Url,
    ) -> Result<AuthorUrl, crate::Error> {
        let url = url.to_string();

        Ok(sqlx::query_as(
            r#"
                INSERT OR IGNORE INTO author_urls (author_id, url) VALUES (?, ?);
                SELECT * FROM author_urls WHERE author_id = ? AND url = ?;
            "#,
        )
        .bind(&self.author_id)
        .bind(&url)
        .bind(&self.author_id)
        .bind(&url)
        .fetch_one(&mut **tx)
        .await?)
    }

    pub async fn add_alias(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        alias: &str,
    ) -> Result<AuthorName, crate::Error> {
        Ok(sqlx::query_as(
            r#"
                INSERT OR IGNORE INTO author_names (author_id, name) VALUES (?, ?);
                SELECT * FROM author_names WHERE author_id = ? AND name = ?;
            "#,
        )
        .bind(&self.author_id)
        .bind(&alias)
        .bind(&self.author_id)
        .bind(&alias)
        .fetch_one(&mut **tx)
        .await?)
    }

    pub async fn get_author_names(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
    ) -> Result<Vec<AuthorName>, crate::Error> {
        Ok(
            sqlx::query_as("SELECT * FROM author_names WHERE author_id = ?;")
                .bind(&self.author_id)
                .fetch_all(&mut **tx)
                .await?,
        )
    }

    pub async fn get_all(tx: &mut Transaction<'_, Sqlite>) -> Result<Vec<Self>, crate::Error> {
        Ok(sqlx::query_as("SELECT * FROM authors;")
            .fetch_all(&mut **tx)
            .await?)
    }
}
