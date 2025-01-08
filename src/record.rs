use std::{
    fs,
    io::{self, BufReader},
    path::PathBuf,
};

use bytemuck::cast;

use crate::{WorkDetails, CONFIG};

pub struct Record {
    pub path: PathBuf,
    pub hash: i32,
    pub details: WorkDetails,
}

impl Record {
    pub fn from_path(path: PathBuf, details: WorkDetails) -> Result<Self, io::Error> {
        let hash = crc32fast::hash(&fs::read(&CONFIG.data_path.join(&path))?);

        Ok(Self {
            path,
            hash: cast(hash),
            details,
        })
    }
}

impl WorkDetails {
    pub fn update(&mut self, other: WorkDetails) {
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
