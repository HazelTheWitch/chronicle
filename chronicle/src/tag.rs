use sqlx::SqlitePool;

use crate::{models::Work, Chronicle};

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

pub async fn tag_tag(
    db: &SqlitePool,
    target: &str,
    tags: impl IntoIterator<Item = impl AsRef<str>>,
) -> Result<(), sqlx::Error> {
    for tag in tags {
        sqlx::query(
            r#"
                INSERT OR IGNORE INTO tags(name) VALUES (?);
                INSERT INTO meta_tags(tag, target) VALUES ((SELECT id FROM tags WHERE name = ?), (SELECT id FROM tags WHERE name = ?));
            "#,
        )
        .bind(tag.as_ref())
        .bind(tag.as_ref())
        .bind(target)
        .execute(db)
        .await?;
    }

    Ok(())
}
