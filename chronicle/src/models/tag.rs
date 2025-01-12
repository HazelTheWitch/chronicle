use crate::id;

use super::work::WorkId;

#[derive(sqlx::FromRow)]
pub struct Tag {
    pub name: String,
    pub discriminator: Option<String>,
    pub id: TagId,
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
