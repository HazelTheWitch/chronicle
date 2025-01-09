use crate::{
    models::{Author, AuthorName},
    Chronicle,
};

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

    pub async fn get_by_name(
        chronicle: &Chronicle,
        name: &str,
    ) -> Result<Vec<Author>, crate::Error> {
        Ok(
            sqlx::query_as("SELECT author_id FROM author_names WHERE name = ?;")
                .bind(name)
                .fetch_all(&chronicle.pool)
                .await?,
        )
    }

    pub async fn get_author_names(
        &self,
        chronicle: &Chronicle,
    ) -> Result<Vec<AuthorName>, crate::Error> {
        Ok(
            sqlx::query_as("SELECT name FROM author_names WHERE author_id = ?;")
                .bind(&self.author_id)
                .fetch_all(&chronicle.pool)
                .await?,
        )
    }
}
