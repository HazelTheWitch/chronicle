use std::{fs, io, path::PathBuf};

use bytemuck::cast;

use crate::Chronicle;

pub struct Record {
    pub path: PathBuf,
    pub hash: i32,
    pub details: RecordDetails,
}

impl Record {
    pub fn from_path(
        chronicle: &Chronicle,
        path: PathBuf,
        details: RecordDetails,
    ) -> Result<Self, io::Error> {
        let hash = crc32fast::hash(&fs::read(&chronicle.config.data_path.join(&path))?);

        Ok(Self {
            path,
            hash: cast(hash),
            details,
        })
    }
}

#[derive(Default, Clone)]
pub struct RecordDetails {
    pub title: Option<String>,
    pub url: Option<url::Url>,
    pub author: Option<String>,
    pub caption: Option<String>,
    pub tags: Vec<String>,
}

impl RecordDetails {
    pub fn update(&mut self, other: RecordDetails) {
        if let Some(title) = other.title {
            self.title = Some(title);
        }
        if let Some(url) = other.url {
            self.url = Some(url);
        }
        if let Some(author) = other.author {
            self.author = Some(author);
        }
        if let Some(caption) = other.caption {
            self.caption = Some(caption);
        }
        self.tags.extend(other.tags);
    }
}
