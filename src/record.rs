use std::path::PathBuf;

use crate::WorkDetails;

pub struct Record {
    pub path: PathBuf,
    pub details: WorkDetails,
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
