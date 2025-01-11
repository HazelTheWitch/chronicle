use std::{convert::Infallible, str::FromStr};

use url::Url;

use crate::{
    models::{Author, AuthorId, AuthorName, AuthorUrl},
    Chronicle,
};

#[derive(Debug, Clone)]
pub enum AuthorQuery {
    Name(String),
    Id(i32),
    Url(url::Url),
}

impl FromStr for AuthorQuery {
    type Err = Infallible;

    fn from_str(query: &str) -> Result<Self, Self::Err> {
        if let Ok(id) = i32::from_str(query) {
            return Ok(Self::Id(id));
        } else if let Ok(url) = url::Url::from_str(query) {
            return Ok(Self::Url(url));
        }

        Ok(Self::Name(query.to_owned()))
    }
}

impl Author {
    pub async fn create(chronicle: &Chronicle, name: &str) -> Result<Author, crate::Error> {
        let author: Author = sqlx::query_as("INSERT INTO authors DEFAULT VALUES RETURNING *;")
            .fetch_one(&chronicle.pool)
            .await?;

        sqlx::query("INSERT INTO author_names(author_id, name) VALUES (?, ?);")
            .bind(author.author_id)
            .bind(name)
            .execute(&chronicle.pool)
            .await?;

        Ok(author)
    }

    pub async fn get_by_id(
        chronicle: &Chronicle,
        author_id: &AuthorId,
    ) -> Result<Option<Author>, crate::Error> {
        Ok(sqlx::query_as("SELECT * FROM authors WHERE author_id = ?;")
            .bind(&author_id)
            .fetch_optional(&chronicle.pool)
            .await?)
    }

    pub async fn get(
        chronicle: &Chronicle,
        query: &AuthorQuery,
    ) -> Result<Vec<Author>, crate::Error> {
        Ok(match query {
            AuthorQuery::Name(name) => {
                sqlx::query_as("SELECT * FROM authors JOIN author_names ON authors.author_id = author_names.author_id WHERE name = ?;")
                    .bind(name)
                    .fetch_all(&chronicle.pool)
                    .await?
            },
            AuthorQuery::Id(id) => {
                sqlx::query_as("SELECT * FROM authors WHERE author_id = ?;")
                    .bind(id)
                    .fetch_all(&chronicle.pool)
                    .await?
            },
            AuthorQuery::Url(url) => {
                sqlx::query_as("SELECT * FROM authors JOIN author_urls ON authors.author_id = author_urls.author_id WHERE author_urls.url = ?;")
                    .bind(url.to_string())
                    .fetch_all(&chronicle.pool)
                    .await?
            },

        })
    }

    pub async fn get_author_urls(
        &self,
        chronicle: &Chronicle,
    ) -> Result<Vec<AuthorUrl>, crate::Error> {
        Ok(
            sqlx::query_as("SELECT * FROM author_urls WHERE author_id = ?;")
                .bind(&self.author_id)
                .fetch_all(&chronicle.pool)
                .await?,
        )
    }

    pub async fn add_url(
        &self,
        chronicle: &Chronicle,
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
        .fetch_one(&chronicle.pool)
        .await?)
    }

    pub async fn add_alias(
        &self,
        chronicle: &Chronicle,
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
        .fetch_one(&chronicle.pool)
        .await?)
    }

    pub async fn get_author_names(
        &self,
        chronicle: &Chronicle,
    ) -> Result<Vec<AuthorName>, crate::Error> {
        Ok(
            sqlx::query_as("SELECT * FROM author_names WHERE author_id = ?;")
                .bind(&self.author_id)
                .fetch_all(&chronicle.pool)
                .await?,
        )
    }

    pub async fn get_all(chronicle: &Chronicle) -> Result<Vec<Self>, crate::Error> {
        Ok(sqlx::query_as("SELECT * FROM authors;")
            .fetch_all(&chronicle.pool)
            .await?)
    }
}
