pub mod bsky;

use anyhow::bail;
use bsky::import_from_bsky;

use crate::{record::Record, WorkDetails};

pub async fn import_from_link(
    db: &sqlx::SqlitePool,
    link: &str,
    provided_details: WorkDetails,
) -> Result<(), anyhow::Error> {
    let url = url::Url::parse(link)?;

    let records = match url.host_str() {
        Some("bsky.app") => import_from_bsky(link).await?,
        _ => bail!("unknown host {link}"),
    };

    for mut record in records {
        record.details.update(provided_details.clone());
        import(db, record).await?;
    }

    Ok(())
}

pub async fn import(db: &sqlx::SqlitePool, record: Record) -> Result<(), sqlx::Error> {
    let tx = db.begin().await?;

    let details = record.details;

    let author_id: Option<i32> = if let Some(author) = &details.author {
        let result: (i32,) = sqlx::query_as(
            r#"
                INSERT OR IGNORE INTO authors(name) VALUES (?);
                SELECT author_id FROM authors WHERE name = ?;
            "#,
        )
        .bind(&author)
        .bind(&author)
        .fetch_one(db)
        .await?;

        Some(result.0)
    } else {
        None
    };

    let work_id: (i32,) = sqlx::query_as(r#"INSERT INTO works(path, url, author_id, title, caption) VALUES (?, ?, ?, ?, ?) RETURNING work_id;"#)
        .bind(&record.path.to_string_lossy())
        .bind(&details.url)
        .bind(&author_id)
        .bind(&details.title)
        .bind(&details.caption)
        .fetch_one(db)
        .await?;

    if details.tags.is_empty() {
        tx.commit().await?;
        return Ok(());
    }

    for tag in details.tags {
        sqlx::query(r#"
                INSERT OR IGNORE INTO tags(name) VALUES (?);
                INSERT INTO work_tags(tag, work_id) VALUES ((SELECT id FROM tags WHERE name = ?), ?);
            "#)
            .bind(&tag)
            .bind(&tag)
            .bind(&work_id.0)
            .execute(db)
            .await?;
    }

    tx.commit().await?;

    Ok(())
}
