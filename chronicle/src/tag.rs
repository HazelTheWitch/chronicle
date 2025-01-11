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

impl TagExpression {
    pub fn approximate_connections(&self) -> usize {
        self.hierarchy
            .windows(2)
            .map(|window| window[0].len() * window[1].len())
            .sum::<usize>()
    }

    pub async fn execute(&self, chronicle: &Chronicle) -> Result<usize, crate::Error> {
        let mut total_connections = 0;

        let tx = chronicle.pool.begin().await?;

        let works = if let Some(query) = &self.query {
            Work::search(&chronicle, query).await?
        } else {
            Vec::new()
        };

        if !works.is_empty() {
            let tags = &self.hierarchy[0];

            for work in &works {
                for tag in tags {
                    if work.tag(&chronicle, tag).await? {
                        total_connections += 1;
                    }
                }
            }
        }

        if self.hierarchy.len() > 1 {
            for window in self.hierarchy.windows(2) {
                let previous_tags = &window[0];
                let next_tags = &window[1];

                for tag in previous_tags {
                    let tag = Tag::get_or_create(&chronicle, &tag).await?;

                    for next in next_tags {
                        if tag.tag(&chronicle, next).await? {
                            total_connections += 1;
                        }
                    }
                }
            }
        }

        tx.commit().await?;

        Ok(total_connections)
    }
}

impl Work {
    pub async fn tag(
        &self,
        chronicle: &Chronicle,
        tag: impl AsRef<str>,
    ) -> Result<bool, crate::Error> {
        let tx = chronicle.pool.begin().await?;

        let success = sqlx::query_as::<_, (i32,)>(r#"
                INSERT OR IGNORE INTO tags(name) VALUES (?);
                INSERT OR IGNORE INTO work_tags(tag, work_id) VALUES ((SELECT id FROM tags WHERE name = ?), ?) RETURNING 1;
            "#)
            .bind(tag.as_ref())
            .bind(tag.as_ref())
            .bind(&self.work_id)
            .fetch_optional(&chronicle.pool)
            .await?
            .is_some();

        tx.commit().await?;

        Ok(success)
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
    ) -> Result<bool, crate::Error> {
        let tx = chronicle.pool.begin().await?;

        let success = sqlx::query_as::<_, (i32,)>(
                r#"
                    INSERT OR IGNORE INTO tags(name) VALUES (?);
                    INSERT OR IGNORE INTO meta_tags(tag, target) VALUES ((SELECT id FROM tags WHERE name = ?), ?) RETURNING 1;
                "#,
            )
            .bind(tag.as_ref())
            .bind(tag.as_ref())
            .bind(&self.id)
            .fetch_optional(&chronicle.pool)
            .await?
            .is_some();

        tx.commit().await?;

        Ok(success)
    }
}
