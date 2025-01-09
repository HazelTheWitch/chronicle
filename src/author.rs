pub async fn get_matching_author_ids(
    db: &sqlx::SqlitePool,
    name: &str,
) -> Result<Vec<i32>, sqlx::Error> {
    Ok(
        sqlx::query_as("SELECT author_id FROM author_names WHERE name = ?;")
            .bind(name)
            .fetch_all(db)
            .await?
            .into_iter()
            .map(|(author_id,)| author_id)
            .collect(),
    )
}

pub async fn get_author_names(
    db: &sqlx::SqlitePool,
    author_id: i32,
) -> Result<Vec<String>, sqlx::Error> {
    Ok(
        sqlx::query_as("SELECT name FROM author_names WHERE author_id = ?;")
            .bind(&author_id)
            .fetch_all(db)
            .await?
            .into_iter()
            .map(|(name,)| name)
            .collect(),
    )
}
