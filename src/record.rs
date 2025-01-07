use std::path::PathBuf;

pub struct Record {
    pub path: PathBuf,
    pub title: Option<String>,
    pub url: Option<String>,
    pub author: Option<String>,
    pub caption: Option<String>,
    pub tags: Vec<String>,
}