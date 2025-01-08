use std::{
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
};

use anyhow::bail;
use atrium_api::{
    agent::{store::MemorySessionStore, AtpAgent},
    types::{TryFromUnknown, Unknown},
    xrpc::http::request,
};
use atrium_xrpc_client::reqwest::ReqwestClient;
use lazy_static::lazy_static;
use regex::Regex;
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;

use crate::{record::Record, WorkDetails, BSKY_IDENTIFIER, BSKY_PASSWORD, CONFIG, SERVICE_NAME};

lazy_static! {
    static ref BSKY_POST_REGEX: Regex = Regex::new(r#"\/profile\/([^\/]+)\/post\/(.+)"#).unwrap();
}

#[derive(Deserialize)]
struct PostRecord {
    pub text: String,
}

pub async fn import_from_bsky(url: &str) -> Result<Vec<Record>, anyhow::Error> {
    let agent = AtpAgent::new(
        ReqwestClient::new("https://bsky.social"),
        MemorySessionStore::default(),
    );

    let identifier = keyring::Entry::new(SERVICE_NAME, BSKY_IDENTIFIER)?.get_password()?;
    let password = keyring::Entry::new(SERVICE_NAME, BSKY_PASSWORD)?.get_password()?;

    agent.login(&identifier, &password).await?;

    let url = url::Url::parse(&url)?;

    let path = url.path();

    let captures = BSKY_POST_REGEX
        .captures(path)
        .ok_or_else(|| anyhow::anyhow!("invalid bsky post {url}"))?;

    let did = captures.get(1).unwrap().as_str();
    let post = captures.get(2).unwrap().as_str();

    let thread = agent
        .api
        .app
        .bsky
        .feed
        .get_post_thread(
            atrium_api::app::bsky::feed::get_post_thread::ParametersData {
                depth: Some(0.try_into().unwrap()),
                parent_height: Some(0.try_into().unwrap()),
                uri: format!("at://{did}/app.bsky.feed.post/{post}"),
            }
            .into(),
        )
        .await?;

    let atrium_api::types::Union::Refs(atrium_api::app::bsky::feed::get_post_thread::OutputThreadRefs::AppBskyFeedDefsThreadViewPost(thread_view_post)) = &thread.thread else {
        bail!("could not retrieve bsky work");
    };

    let post = &thread_view_post.post;

    let deserialized_post = PostRecord::try_from_unknown(post.record.clone())?;

    let caption = deserialized_post.text;

    let author = post.author.handle.clone().as_str().to_owned();

    let work_details = WorkDetails {
        tags: Vec::new(),
        title: None,
        author: Some(author.to_string()),
        url: Some(url.to_string()),
        caption: Some(caption),
    };

    let mut records = Vec::new();

    if let Some(atrium_api::types::Union::Refs(
        atrium_api::app::bsky::feed::defs::PostViewEmbedRefs::AppBskyEmbedImagesView(view),
    )) = &post.embed
    {
        for image_url in view.images.iter().map(|data| data.fullsize.clone()) {
            let file_name = format!("{}.jpg", Uuid::new_v4());
            let mut writer = BufWriter::new(File::create_new(CONFIG.data_path.join(&file_name))?);

            let request = reqwest::get(&image_url).await?;

            let data = request.bytes().await?;

            writer.write_all(&data)?;

            records.push(Record {
                path: PathBuf::from(file_name),
                hash: bytemuck::cast(crc32fast::hash(&data)),
                details: work_details.clone(),
            });
        }
    }

    if records.is_empty() {
        bail!("No embedded works on bsky post.");
    }

    Ok(records)
}
