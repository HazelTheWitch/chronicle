use std::{
    collections::HashMap,
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
    sync::Arc,
};

use async_trait::async_trait;
use atrium_api::{
    agent::{store::MemorySessionStore, AtpAgent, Session},
    app::bsky::feed::{defs::PostViewEmbedRefs, get_post_thread},
    types::{TryFromUnknown, Union},
};
use atrium_xrpc_client::reqwest::ReqwestClient;
use lazy_static::lazy_static;
use nom::{
    bytes::complete::{tag, take_while},
    combinator::recognize,
    IResult,
};
use regex::Regex;
use serde::Deserialize;
use tokio::sync::{OnceCell, RwLock};
use tracing::{error, warn};
use uuid::Uuid;

use crate::{
    author::AuthorQuery,
    record::{Record, RecordDetails},
    ServiceError,
};

use super::Service;

lazy_static! {
    static ref BSKY_POST_REGEX: Regex = Regex::new(r#"\/profile\/([^\/]+)\/post\/(.+)"#).unwrap();
}

struct BskyPath<'s> {
    pub did: &'s str,
    pub post: &'s str,
}

fn parse_bsky_path(i: &str) -> IResult<&str, BskyPath<'_>> {
    let (i, _) = tag(r#"/profile/"#)(i)?;
    let (i, did) = recognize(take_while(|c| c != '/'))(i)?;
    let (i, _) = tag(r#"/post/"#)(i)?;
    let (i, post) = recognize(take_while(|c| c != '/'))(i)?;

    Ok((i, BskyPath { did, post }))
}

const BSKY_IDENTIFIER: &str = "bsky-identifier";
const BSKY_PASSWORD: &str = "bsky-password";

static AGENT: OnceCell<AtpAgent<MemorySessionStore, ReqwestClient>> = OnceCell::const_new();

pub struct Bsky;

#[async_trait]
impl Service for Bsky {
    fn host(&self) -> &str {
        "bsky.app"
    }

    fn secrets(&self) -> &[&str] {
        &[BSKY_IDENTIFIER, BSKY_PASSWORD]
    }

    async fn import(
        &self,
        chronicle: &crate::Chronicle,
        url: url::Url,
        records: &mut Vec<Record>,
        secrets: Arc<RwLock<HashMap<String, String>>>,
    ) -> Result<(), crate::Error> {
        let agent = AGENT
            .get_or_try_init::<crate::Error, _, _>(|| async move {
                let agent = AtpAgent::new(
                    ReqwestClient::new("https://bsky.social"),
                    MemorySessionStore::default(),
                );

                let secrets = secrets.read().await;

                let identifier = secrets[BSKY_IDENTIFIER].as_str();
                let password = secrets[BSKY_PASSWORD].as_str();

                agent
                    .login(&identifier, &password)
                    .await
                    .map_err(ServiceError::from)?;

                Ok(agent)
            })
            .await?;

        let (_, BskyPath { did, post }) =
            parse_bsky_path(&url.path()).map_err(|_| crate::Error::InvalidUrl {
                service: "bsky.app",
                url: url.clone(),
            })?;

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
            .await
            .map_err(ServiceError::from)?;

        let Union::Refs(get_post_thread::OutputThreadRefs::AppBskyFeedDefsThreadViewPost(
            thread_view_post,
        )) = &thread.thread
        else {
            error!("Could not access bsky post at {url}");
            return Ok(());
        };

        let post = &thread_view_post.post;

        let deserialized_post = PostRecord::try_from_unknown(post.record.clone())
            .map_err(|err| crate::Error::Generic(format!("could not parse bsky post: {err}")))?;

        let caption = deserialized_post.text;

        let author = post.author.handle.clone().as_str().to_owned();

        let details = RecordDetails {
            tags: Vec::new(),
            title: None,
            author: Some(AuthorQuery::Name(author.to_string())),
            url: Some(url),
            caption: Some(caption),
        };

        match &post.embed {
            Some(Union::Refs(PostViewEmbedRefs::AppBskyEmbedImagesView(view))) => {
                for image_url in view.images.iter().map(|data| data.fullsize.clone()) {
                    let file_name = format!("{}.jpg", Uuid::new_v4());
                    let mut writer = BufWriter::new(File::create_new(
                        chronicle.config.data_path.join(&file_name),
                    )?);

                    let request = reqwest::get(&image_url).await?;

                    let data = request.bytes().await?;

                    writer.write_all(&data)?;

                    records.push(Record {
                        path: PathBuf::from(file_name),
                        hash: bytemuck::cast(crc32fast::hash(&data)),
                        size: data.len(),
                        details: details.clone(),
                    });
                }
            }
            _ => {
                warn!("This post type can not be imported");
            }
        }

        Ok(())
    }
}

#[derive(Deserialize)]
struct PostRecord {
    pub text: String,
}
