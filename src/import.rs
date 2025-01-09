pub mod bsky;

use anyhow::bail;
use bsky::import_from_bsky;
use tracing::info;

use crate::{author::get_matching_author_ids, record::Record, tag::tag_work, WorkDetails};

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

pub async fn import(db: &sqlx::SqlitePool, record: Record) -> anyhow::Result<()> {
    let tx = db.begin().await?;

    let details = record.details;

    let author_id = if let Some(author) = &details.author {
        let mut all_author_ids = get_matching_author_ids(&db, &author).await?;

        if all_author_ids.is_empty() {
            let author_id: (i32,) =
                sqlx::query_as("INSERT INTO authors DEFAULT VALUES RETURNING author_id;")
                    .fetch_one(db)
                    .await?;
            sqlx::query("INSERT INTO author_names (author_id, name) VALUES (?, ?);")
                .bind(&author_id.0)
                .bind(&author)
                .execute(db)
                .await?;
            Some(author_id.0)
        } else if all_author_ids.len() == 1 {
            Some(all_author_ids.remove(0))
        } else {
            bail!("multiple authors use this name");
        }
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
