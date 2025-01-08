use sqlx::SqlitePool;

pub async fn tag_work(
    db: &SqlitePool,
    work_id: i32,
    tags: impl IntoIterator<Item = impl AsRef<str>>,
) -> Result<(), sqlx::Error> {
    for tag in tags {
        sqlx::query(r#"
                INSERT OR IGNORE INTO tags(name) VALUES (?);
                INSERT INTO work_tags(tag, work_id) VALUES ((SELECT id FROM tags WHERE name = ?), ?);
            "#)
            .bind(tag.as_ref())
            .bind(tag.as_ref())
            .bind(work_id)
            .execute(db)
            .await?;
    }

    Ok(())
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
