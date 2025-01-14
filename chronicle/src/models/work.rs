use sqlx::{query_as, Sqlite, Transaction};

use crate::id;

use super::{author::AuthorId, Tag};

#[derive(serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Work {
    pub path: String,
    pub work_id: WorkId,
    pub size: u64,
    pub title: Option<String>,
    pub author_id: Option<AuthorId>,
    pub caption: Option<String>,
    pub url: Option<String>,
    pub hash: i32,
}

impl Work {
    pub async fn get_tags(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
    ) -> Result<Vec<Tag>, crate::Error> {
        Ok(
            query_as("SELECT * FROM tags JOIN work_tags ON work_tags.work_id = ?;")
                .bind(&self.work_id)
                .fetch_all(&mut **tx)
                .await?,
        )
    }

    pub async fn remove_tag(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        tag: &Tag,
    ) -> Result<(), crate::Error> {
        sqlx::query("DELETE FROM work_tags WHERE work_id = ? AND tag = ?;")
            .bind(&self.work_id)
            .bind(&tag.id)
            .execute(&mut **tx)
            .await?;

        Ok(())
    }

    pub async fn update(&self, tx: &mut Transaction<'_, Sqlite>) -> Result<(), crate::Error> {
        sqlx::query("UPDATE works SET size = ?, title = ?, author_id = ?, caption = ?, url = ?, hash = ? WHERE work_id = ?")
            .bind(&(self.size as i64))
            .bind(&self.title)
            .bind(&self.author_id)
            .bind(&self.caption)
            .bind(&self.url)
            .bind(&self.hash)
            .bind(&self.work_id)
            .execute(&mut **tx)
            .await?;

        Ok(())
    }
}

id!(Work);
