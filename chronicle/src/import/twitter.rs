use std::{
    collections::HashMap,
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
};

use async_trait::async_trait;
use reqwest::header::USER_AGENT;
use serde::Deserialize;
use url::Url;
use uuid::Uuid;

use crate::{
    author::AuthorQuery,
    record::{Record, RecordDetails},
    ServiceError,
};

use super::Service;

pub struct Twitter;

#[async_trait]
impl Service for Twitter {
    fn host_matches(&self, host: &str) -> bool {
        host == "twitter.com" || host == "x.com"
    }
    fn name(&self) -> &str {
        "twitter"
    }
    fn secrets(&self) -> &[&str] {
        &[]
    }
    async fn authenticate(
        &self,
        _: &HashMap<String, String>,
        _: Option<HashMap<String, String>>,
    ) -> Result<HashMap<String, String>, crate::Error> {
        Ok(HashMap::new())
    }
    async fn import(
        &self,
        chronicle: &crate::Chronicle,
        url: Url,
        records: &mut Vec<Record>,
        _: HashMap<String, String>,
        _: HashMap<String, String>,
    ) -> Result<(), crate::Error> {
        let mut fixtweet_url = url.clone();
        fixtweet_url.set_host(Some("api.fxtwitter.com"))?;

        let client = reqwest::Client::new();

        let response: FixTweetResponse = client
            .get(fixtweet_url)
            .header(USER_AGENT, "Chronicle")
            .send()
            .await?
            .json()
            .await?;

        if response.code != 200 {
            return Err(crate::Error::Service(ServiceError::Twitter(response.code)));
        }

        let details = RecordDetails {
            title: None,
            author: Some(AuthorQuery::Name(response.tweet.author.screen_name.clone())),
            author_url: Some(response.tweet.author.url),
            tags: Vec::new(),
            url: Some(response.tweet.url),
            caption: Some(response.tweet.text),
        };

        for photo in response.tweet.media.photos.iter().flatten() {
            let file_name = format!("{}.jpg", Uuid::new_v4());
            let mut writer = BufWriter::new(File::create_new(
                chronicle.config.data_path.join(&file_name),
            )?);

            let request = reqwest::get(photo.url.clone()).await?;

            let data = request.bytes().await?;

            writer.write_all(&data)?;

            records.push(Record {
                path: PathBuf::from(file_name),
                size: data.len(),
                hash: bytemuck::cast(crc32fast::hash(&data)),
                details: details.clone(),
            });
        }

        Ok(())
    }
}

#[derive(Deserialize)]
struct FixTweetResponse {
    pub code: u32,
    pub tweet: Tweet,
}

#[derive(Deserialize)]
struct Tweet {
    pub url: Url,
    pub text: String,
    pub author: TweetAuthor,
    pub media: TweetMedia,
}

#[derive(Deserialize)]
struct TweetAuthor {
    pub screen_name: String,
    pub url: Url,
}

#[derive(Deserialize)]
struct TweetMedia {
    pub photos: Option<Vec<TweetPhoto>>,
}

#[derive(Deserialize)]
struct TweetPhoto {
    pub url: Url,
}
