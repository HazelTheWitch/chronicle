use std::fmt::Display;

use crate::id;

use super::work::WorkId;

#[derive(sqlx::FromRow)]
pub struct Tag {
    pub name: String,
    pub discriminator: Option<String>,
    pub id: TagId,
}

impl Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)?;

        if let Some(discriminator) = &self.discriminator {
            write!(f, "#{discriminator}")?;
        }

        Ok(())
    }
}

#[derive(sqlx::FromRow)]
pub struct WorkTag {
    pub tag: TagId,
    pub work_id: WorkId,
}

#[derive(sqlx::FromRow)]
pub struct MetaTag {
    pub tag: TagId,
    pub target: TagId,
}

id!(Tag);
