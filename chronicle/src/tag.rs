use sqlx::SqlitePool;

use crate::{
    models::{Tag, Work},
    Chronicle,
};

impl Work {
    pub async fn tag(
        &self,
        chronicle: &Chronicle,
        tag: impl AsRef<str>,
    ) -> Result<(), crate::Error> {
        let tx = chronicle.pool.begin().await?;

        sqlx::query(r#"
                INSERT OR IGNORE INTO tags(name) VALUES (?);
                INSERT INTO work_tags(tag, work_id) VALUES ((SELECT id FROM tags WHERE name = ?), ?);
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

    pub async fn create(chronicle: &Chronicle, name: &str) -> Result<Tag, crate::Error> {
        Ok(sqlx::query_as(
            r#"
            INSERT OR IGNORE INTO tags (name) VALUES (?);
            SELECT * FROM tags WHERE id = ?;
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
                    INSERT INTO meta_tags(tag, target) VALUES ((SELECT id FROM tags WHERE name = ?), (SELECT id FROM tags WHERE name = ?));
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
