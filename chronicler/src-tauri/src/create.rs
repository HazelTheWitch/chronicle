use std::path::PathBuf;

use chronicle::{
    models::{AuthorId, WorkId},
    tag::DiscriminatedTag,
};
use serde::{Deserialize, Serialize};
use tauri::Url;

#[derive(Serialize, Deserialize)]
pub struct AuthorCreate {
    pub urls: Vec<String>,
    pub names: Vec<String>,
    pub id: Option<AuthorId>,
}

#[derive(Serialize, Deserialize)]
pub struct WorkCreate {
    pub path: PathBuf,
    pub title: Option<String>,
    pub author: Option<AuthorCreate>,
    pub caption: Option<String>,
    pub url: Option<Url>,
    pub tags: Vec<DiscriminatedTag>,
}

#[derive(Serialize, Deserialize)]
pub struct WorkEdit {
    #[serde(flatten)]
    pub create: WorkCreate,
    pub work_id: WorkId,
}
