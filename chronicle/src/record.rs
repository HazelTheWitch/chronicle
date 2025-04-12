use std::{fs, io, path::PathBuf};

use bytemuck::cast;

use crate::{author::AuthorQuery, tag::DiscriminatedTag, Chronicle};

pub struct Record {
    pub path: PathBuf,
    pub size: usize,
    pub hash: i32,
    pub details: RecordDetails,
}

impl Record {
    pub fn from_path(
        chronicle: &Chronicle,
        path: PathBuf,
        details: RecordDetails,
    ) -> Result<Self, io::Error> {
        let data = fs::read(&chronicle.config.data_path.join(&path))?;
        let hash = crc32fast::hash(&data);
        let size = data.len();

        Ok(Self {
            path,
            size,
            hash: cast(hash),
            details,
        })
    }
}

#[derive(Default, Debug, Clone)]
pub struct RecordDetails {
    pub title: Option<String>,
    pub url: Option<url::Url>,
    pub author: Option<AuthorQuery>,
    pub author_url: Option<url::Url>,
    pub caption: Option<String>,
    pub tags: Vec<DiscriminatedTag>,
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
        if let Some(author_url) = other.author_url {
            self.author_url = Some(author_url);
        }
        self.tags.extend(other.tags);
    }
}
