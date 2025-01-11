mod builder;
mod parse;

use std::str::FromStr;

use parse::tag_expression;

use crate::{
    models::{Tag, Work},
    parse::ParseError,
    search::Query,
    Chronicle,
};

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct TagExpression {
    pub query: Option<Query>,
    pub hierarchy: Vec<Vec<String>>,
}

impl TagExpression {
    pub fn new(
        query: Option<Query>,
        hierarchy: impl IntoIterator<Item = impl IntoIterator<Item = impl Into<String>>>,
    ) -> Self {
        Self {
            query,
            hierarchy: hierarchy
                .into_iter()
                .map(|i| i.into_iter().map(|s| s.into()).collect())
                .collect(),
        }
    }
}

impl FromStr for TagExpression {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (i, expression) = tag_expression(s)?;

        if !i.is_empty() {
            return Err(ParseError::ParserDidNotFinish(i.to_owned()));
        }

        Ok(expression)
    }
}

impl Work {
    pub async fn tag(
        &self,
        chronicle: &Chronicle,
        tag: impl AsRef<str>,
    ) -> Result<(), crate::Error> {
        let tx = chronicle.pool.begin().await?;

        sqlx::query(r#"
                INSERT OR IGNORE INTO tags(name) VALUES (?);
                INSERT OR IGNORE INTO work_tags(tag, work_id) VALUES ((SELECT id FROM tags WHERE name = ?), ?);
            "#)
            .bind(tag.as_ref())
            .bind(tag.as_ref())
            .bind(&self.work_id)
            .execute(&chronicle.pool)
            .await?;

        tx.commit().await?;

        Ok(())
    }
}

impl Tag {
    pub async fn get_by_name(
        chronicle: &Chronicle,
        name: &str,
    ) -> Result<Option<Tag>, crate::Error> {
        Ok(sqlx::query_as("SELECT * FROM tags WHERE id = ;")
            .bind(name)
            .fetch_optional(&chronicle.pool)
            .await?)
    }

    pub async fn get_or_create(chronicle: &Chronicle, name: &str) -> Result<Tag, crate::Error> {
        Ok(sqlx::query_as(
            r#"
            INSERT OR IGNORE INTO tags (name) VALUES (?);
            SELECT * FROM tags WHERE name = ?;
        "#,
        )
        .bind(name)
        .bind(name)
        .fetch_one(&chronicle.pool)
        .await?)
    }

    pub async fn tag(
        &self,
        chronicle: &Chronicle,
        tag: impl AsRef<str>,
    ) -> Result<(), crate::Error> {
        let tx = chronicle.pool.begin().await?;

        sqlx::query(
                r#"
                    INSERT OR IGNORE INTO tags(name) VALUES (?);
                    INSERT OR IGNORE INTO meta_tags(tag, target) VALUES ((SELECT id FROM tags WHERE name = ?), ?);
                "#,
            )
            .bind(tag.as_ref())
            .bind(tag.as_ref())
            .bind(&self.id)
            .execute(&chronicle.pool)
            .await?;

        tx.commit().await?;

        Ok(())
    }
}
