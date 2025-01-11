use crate::id;

use super::author::AuthorId;

#[derive(sqlx::FromRow)]
pub struct Work {
    pub path: String,
    pub work_id: WorkId,
    pub size: u64,
    pub title: Option<String>,
    pub author_id: Option<AuthorId>,
    pub caption: Option<String>,
    pub url: Option<String>,
    pub hash: i32,
}

id!(Work);
