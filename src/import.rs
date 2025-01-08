pub mod bsky;

use anyhow::bail;
use bsky::import_from_bsky;

use crate::{record::Record, tag::tag_work, WorkDetails};

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

pub async fn work_present_with_link(
    db: &sqlx::SqlitePool,
    link: &str,
) -> Result<bool, sqlx::Error> {
    Ok(sqlx::query(r#"SELECT 1 FROM works WHERE url = ? LIMIT 1;"#)
        .bind(link)
        .fetch_optional(db)
        .await?
        .is_some())
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

    let work_id: (i32,) = sqlx::query_as(r#"INSERT INTO works(path, url, author_id, title, caption, hash) VALUES (?, ?, ?, ?, ?, ?) RETURNING work_id;"#)
        .bind(&record.path.to_string_lossy())
        .bind(&details.url)
        .bind(&author_id)
        .bind(&details.title)
        .bind(&details.caption)
        .bind(&record.hash)
        .fetch_one(db)
        .await?;

    tag_work(db, work_id.0, details.tags).await?;

    tx.commit().await?;

    Ok(())
}
