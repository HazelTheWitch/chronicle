use crate::id;

#[derive(serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Author {
    pub author_id: AuthorId,
}
#[derive(serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct AuthorName {
    pub author_id: AuthorId,
    pub name: String,
}

#[derive(serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct AuthorUrl {
    pub author_id: AuthorId,
    pub url: String,
}

id!(Author);
