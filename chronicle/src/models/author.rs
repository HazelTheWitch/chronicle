use crate::id;

#[derive(sqlx::FromRow)]
pub struct Author {
    pub author_id: AuthorId,
}

#[derive(sqlx::FromRow)]
pub struct AuthorName {
    pub author_id: AuthorId,
    pub name: String,
}

#[derive(sqlx::FromRow)]
pub struct AuthorUrl {
    pub author_id: AuthorId,
    pub url: String,
}

id!(Author);
