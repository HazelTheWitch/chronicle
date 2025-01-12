pub mod parse;

use std::{fmt::Display, str::FromStr};

use parse::{discriminated_tag, tag_expression, ParsedTag};
use sqlx::{Acquire, Sqlite, Transaction};

use crate::{
    models::{ModelKind, Tag, Work},
    parse::ParseError,
    search::Query,
    Chronicle,
};

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub struct DiscriminatedTag {
    pub name: String,
    pub discriminator: Option<String>,
}

impl<'s> From<ParsedTag<'s>> for DiscriminatedTag {
    fn from(
        ParsedTag {
            name,
            discriminator,
        }: ParsedTag,
    ) -> Self {
        Self {
            name: name.to_owned(),
            discriminator: discriminator.map(String::from),
        }
    }
}

impl FromStr for DiscriminatedTag {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (left, parsed_tag) = discriminated_tag(s)?;

        if !left.is_empty() {
            return Err(ParseError::ParserDidNotFinish(left.to_string()));
        }

        Ok(parsed_tag.into())
    }
}

impl Display for DiscriminatedTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.name)?;

        if let Some(discriminator) = &self.discriminator {
            write!(f, "#{discriminator}")?;
        }

        Ok(())
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct TagExpression {
    pub query: Option<Query>,
    pub hierarchy: Vec<Vec<DiscriminatedTag>>,
}

impl TagExpression {
    pub fn new(
        query: Option<Query>,
        hierarchy: impl IntoIterator<Item = impl IntoIterator<Item = impl Into<DiscriminatedTag>>>,
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

    pub async fn create_missing_tags(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
    ) -> Result<usize, crate::Error> {
        let mut created = 0;

        let mut tx = tx.begin().await?;

        for level in &self.hierarchy {
            for DiscriminatedTag {
                name,
                discriminator,
            } in level.iter()
            {
                if Tag::try_get_discriminated(&mut tx, name, discriminator.as_deref())
                    .await?
                    .is_none()
                {
                    Tag::create(&mut tx, name, discriminator.as_deref()).await?;
                    created += 1;
                }
            }
        }

        tx.commit().await?;

        Ok(created)
    }

    pub async fn list_tags(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
    ) -> Result<Vec<Tag>, crate::Error> {
        let mut tags = Vec::new();

        for level in &self.hierarchy {
            for DiscriminatedTag {
                name,
                discriminator,
            } in level.iter()
            {
                tags.push(
                    Tag::get_discriminated(tx, name.as_str(), discriminator.as_deref()).await?,
                );
            }
        }

        Ok(tags)
    }

    pub async fn execute(&self, tx: &mut Transaction<'_, Sqlite>) -> Result<usize, crate::Error> {
        let mut total_connections = 0;

        let mut tx = tx.begin().await?;

        let works = if let Some(query) = &self.query {
            Work::search(&mut tx, query).await?
        } else {
            Vec::new()
        };

        if !works.is_empty() {
            let tags = &self.hierarchy[0];

            for work in &works {
                for DiscriminatedTag {
                    name,
                    discriminator,
                } in tags
                {
                    let tag =
                        Tag::get_discriminated_or_create(&mut tx, name, discriminator.as_deref())
                            .await?;

                    if work.tag(&mut tx, &tag).await? {
                        total_connections += 1;
                    }
                }
            }
        }

        if self.hierarchy.len() > 1 {
            for window in self.hierarchy.windows(2) {
                let previous_tags = &window[0];
                let next_tags = &window[1];

                for DiscriminatedTag {
                    name,
                    discriminator,
                } in previous_tags
                {
                    let tag =
                        Tag::get_discriminated_or_create(&mut tx, name, discriminator.as_deref())
                            .await?;

                    for DiscriminatedTag {
                        name,
                        discriminator,
                    } in next_tags
                    {
                        let next = Tag::get_discriminated_or_create(
                            &mut tx,
                            name,
                            discriminator.as_deref(),
                        )
                        .await?;
                        if tag.tag(&mut tx, &next).await? {
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
        tx: &mut Transaction<'_, Sqlite>,
        tag: &Tag,
    ) -> Result<bool, crate::Error> {
        Ok(sqlx::query_as::<_, (i32,)>(
            "INSERT OR IGNORE INTO work_tags(tag, work_id) VALUES (?, ?) RETURNING 1;",
        )
        .bind(&tag.id)
        .bind(&self.work_id)
        .fetch_optional(&mut **tx)
        .await?
        .is_some())
    }
}

impl Tag {
    pub async fn try_get_discriminated(
        tx: &mut Transaction<'_, Sqlite>,
        name: &str,
        discriminator: Option<&str>,
    ) -> Result<Option<Tag>, crate::Error> {
        if let Some(discriminator) = discriminator {
            Ok(
                sqlx::query_as("SELECT * FROM tags WHERE name = ? AND discriminator = ?;")
                    .bind(name)
                    .bind(discriminator)
                    .fetch_optional(&mut **tx)
                    .await?,
            )
        } else {
            Ok(
                sqlx::query_as("SELECT * FROM tags WHERE name = ? AND discriminator IS NULL;")
                    .bind(name)
                    .fetch_optional(&mut **tx)
                    .await?,
            )
        }
    }

    pub async fn get_discriminated(
        tx: &mut Transaction<'_, Sqlite>,
        name: &str,
        discriminator: Option<&str>,
    ) -> Result<Tag, crate::Error> {
        Self::try_get_discriminated(tx, name, discriminator)
            .await?
            .ok_or(crate::Error::NotFound {
                kind: ModelKind::Tag,
            })
    }

    pub async fn get(
        tx: &mut Transaction<'_, Sqlite>,
        name: &str,
    ) -> Result<Vec<Tag>, crate::Error> {
        Ok(sqlx::query_as("SELECT * FROM tags WHERE name = ?;")
            .bind(name)
            .fetch_all(&mut **tx)
            .await?)
    }

    pub async fn get_discriminated_or_create(
        tx: &mut Transaction<'_, Sqlite>,
        name: &str,
        discriminator: Option<&str>,
    ) -> Result<Tag, crate::Error> {
        let Some(tag) = Self::try_get_discriminated(tx, name, discriminator).await? else {
            return Self::create(tx, name, discriminator).await;
        };

        Ok(tag)
    }

    pub async fn create(
        tx: &mut Transaction<'_, Sqlite>,
        name: &str,
        discriminator: Option<&str>,
    ) -> Result<Tag, crate::Error> {
        Ok(
            sqlx::query_as("INSERT INTO tags(name, discriminator) VALUES (?, ?) RETURNING *;")
                .bind(name)
                .bind(discriminator)
                .fetch_one(&mut **tx)
                .await?,
        )
    }

    pub async fn tag(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        tag: &Self,
    ) -> Result<bool, crate::Error> {
        Ok(sqlx::query_as::<_, (i32,)>(
            "INSERT OR IGNORE INTO meta_tags(tag, target) VALUES (?, ?) RETURNING 1;",
        )
        .bind(&tag.id)
        .bind(&self.id)
        .fetch_optional(&mut **tx)
        .await?
        .is_some())
    }
}
