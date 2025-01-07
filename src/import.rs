pub mod bsky;

use crate::record::Record;

pub async fn import_from_link(link: &str) -> Result<(), anyhow::Error> {
    Ok(())
}

pub async fn import(db: &sqlx::SqlitePool, record: Record) -> Result<(), sqlx::Error> {
    let author_id: Option<i32> = if let Some(author) = &record.author {
        sqlx::query(r#"INSERT OR IGNORE INTO "authors"("name") VALUES (?);"#)
            .bind(&author)
            .execute(db)
            .await?;

        let author_id: (i32,) =
            sqlx::query_as(r#"SELECT "author_id" FROM "authors" WHERE "name" = ? LIMIT 1;"#)
                .bind(&author)
                .fetch_one(db)
                .await?;

        Some(author_id.0)
    } else {
        None
    };

    let work_id: (i32,) = sqlx::query_as(r#"INSERT INTO "works" ("path", "url", "author_id", "title", "caption") VALUES (?, ?, ?, ?, ?) RETURNING "work_id";"#)
        .bind(&record.path.to_string_lossy())
        .bind(&record.url)
        .bind(&author_id)
        .bind(&record.title)
        .bind(&record.caption)
        .fetch_one(db)
        .await?;

    if record.tags.is_empty() {
        return Ok(());
    }

    let mut query_builder = sqlx::QueryBuilder::new(r#"INSERT OR IGNORE INTO "tags"("name") "#);

    query_builder.push_values(record.tags.iter(), |mut b, tag| {
        b.push_bind(tag);
    });

    let query = query_builder.build();

    query.execute(db).await?;

    let mut query_builder =
        sqlx::QueryBuilder::new(r#"INSERT INTO "work_tags"("tag", "work_id") "#);

    query_builder.push_values(record.tags.iter(), |mut b, tag| {
        b.push_bind(tag).push_bind(&work_id.0);
    });

    let query = query_builder.build();

    query.execute(db).await?;

    Ok(())
}
