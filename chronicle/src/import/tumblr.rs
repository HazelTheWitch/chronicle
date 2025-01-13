use std::{
    collections::HashMap,
    ffi::OsString,
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
    sync::Arc,
};

use async_trait::async_trait;
use nom::{
    bytes::complete::tag, character::complete::anychar, combinator::recognize, multi::many1,
    sequence::preceded, IResult,
};
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthUrl, AuthorizationCode, ClientId,
    ClientSecret, CsrfToken, PkceCodeChallenge, RefreshToken, Scope, TokenResponse, TokenUrl,
};
use regex::Regex;
use serde::Deserialize;
use tokio::sync::{watch, OnceCell, RwLock};
use url::Url;
use uuid::Uuid;

use crate::{
    author::AuthorQuery,
    http::register_oauth2_handler,
    record::{Record, RecordDetails},
    HTTP_CLIENT,
};

use super::Service;

const TUMBLR_CONSUMER: &str = "tumblr-consumer";
const TUMBLR_SECRET: &str = "tumblr-secret";

const ACCESS_TOKEN: &str = "access-token";
const REFRESH_TOKEN: &str = "refresh-token";

lazy_static::lazy_static! {
    static ref TUMBLR_HOST: Regex = Regex::new(r#"^[^\.]+\.tumblr\.com$"#).unwrap();
}

#[derive(Default)]
pub struct Tumblr {
    pub oauth_reciever: OnceCell<RwLock<watch::Receiver<AuthorizationCode>>>,
}

fn tumblr_id_from_path(i: &str) -> IResult<&str, TumblrId> {
    let (i, blog) = preceded(
        tag("/"),
        recognize(many1(preceded(nom::combinator::not(tag("/")), anychar))),
    )(i)?;
    let (i, post_id) = preceded(
        tag("/"),
        recognize(many1(preceded(nom::combinator::not(tag("/")), anychar))),
    )(i)?;

    Ok((
        i,
        TumblrId {
            blog: blog.to_owned(),
            post_id: post_id.to_owned(),
        },
    ))
}

struct TumblrId {
    blog: String,
    post_id: String,
}

#[derive(Debug, Deserialize)]
struct TumblrResponse {
    response: NpfPost,
}

#[serde_with::serde_as]
#[derive(Deserialize, Debug)]
struct NpfPost {
    #[serde_as(as = "serde_with::VecSkipError<_>")]
    content: Vec<Content>,
    blog: Blog,
    post_url: String,
    summary: String,
}

#[derive(Deserialize, Debug)]
struct Blog {
    name: String,
    url: String,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Content {
    Text { text: String },
    Image { media: Vec<Media> },
}

#[derive(Deserialize, Debug)]
struct Media {
    url: String,
    width: u32,
    height: u32,
}

#[async_trait]
impl Service for Tumblr {
    fn host_matches(&self, host: &str) -> bool {
        TUMBLR_HOST.is_match(host)
    }

    fn name(&self) -> &str {
        "tumblr"
    }

    fn secrets(&self) -> &[&str] {
        &[TUMBLR_CONSUMER, TUMBLR_SECRET]
    }

    async fn authenticate(
        &self,
        secrets: &HashMap<String, String>,
        previous_result: Option<HashMap<String, String>>,
    ) -> Result<HashMap<String, String>, crate::Error> {
        let rx = self
            .oauth_reciever
            .get_or_init(|| register_oauth2_handler(String::from("tumblr")))
            .await;

        let client = BasicClient::new(
            ClientId::new(secrets[TUMBLR_CONSUMER].to_string()),
            Some(ClientSecret::new(secrets[TUMBLR_SECRET].to_string())),
            AuthUrl::new("https://www.tumblr.com/oauth2/authorize".to_string()).unwrap(),
            Some(TokenUrl::new("https://api.tumblr.com/v2/oauth2/token".to_string()).unwrap()),
        );

        if let Some(mut previous) = previous_result {
            if let Some(refresh) = previous.get(REFRESH_TOKEN) {
                let response = client
                    .exchange_refresh_token(&RefreshToken::new(refresh.to_owned()))
                    .add_scope(Scope::new("basic".to_string()))
                    .add_scope(Scope::new("offline_access".to_string()))
                    .request_async(async_http_client)
                    .await
                    .map_err(|err| crate::Error::Oauth2(Box::new(err)))?;

                previous.insert(
                    ACCESS_TOKEN.to_string(),
                    response.access_token().secret().to_owned(),
                );

                if let Some(refresh) = response.refresh_token() {
                    previous.insert(REFRESH_TOKEN.to_string(), refresh.secret().to_owned());
                }

                return Ok(previous);
            }
        }
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        let (auth_url, _) = client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("basic".to_string()))
            .add_scope(Scope::new("offline_access".to_string()))
            .set_pkce_challenge(pkce_challenge)
            .url();

        open::that(OsString::from(auth_url.to_string()))?;

        let auth_code = {
            let mut rx = rx.write().await;

            rx.changed()
                .await
                .map_err(|err| crate::Error::Generic(err.to_string()))?;

            let auth_code = rx.borrow();

            auth_code.to_owned()
        };

        let token_result = client
            .exchange_code(auth_code)
            .set_pkce_verifier(pkce_verifier)
            .request_async(async_http_client)
            .await
            .map_err(|err| crate::Error::Oauth2(Box::new(err)))?;

        let mut results = HashMap::new();

        results.insert(
            ACCESS_TOKEN.to_string(),
            token_result.access_token().secret().to_owned(),
        );

        if let Some(refresh_token) = token_result.refresh_token() {
            results.insert(REFRESH_TOKEN.to_string(), refresh_token.secret().to_owned());
        }

        Ok(results)
    }

    async fn import(
        &self,
        chronicle: &crate::Chronicle,
        url: Url,
        records: &mut Vec<Record>,
        secrets: HashMap<String, String>,
        authentication: HashMap<String, String>,
    ) -> Result<(), crate::Error> {
        let TumblrId { blog, post_id } = if url.host_str() == Some("www.tumblr.com") {
            tumblr_id_from_path(url.path())
                .map_err(|_| crate::Error::InvalidUrl {
                    service: "tumblr",
                    url: url.clone(),
                })?
                .1
        } else {
            let Some(host_str) = url.host_str() else {
                return Err(crate::Error::InvalidUrl {
                    service: "tumblr",
                    url: url.clone(),
                });
            };

            let blog = host_str.trim_end_matches(".tumblr.com");
            let post_id = url.path().trim_start_matches("/post/");

            TumblrId {
                blog: blog.to_string(),
                post_id: post_id.to_string(),
            }
        };

        let url = format!("https://api.tumblr.com/v2/blog/{blog}/posts/{post_id}");

        let response = HTTP_CLIENT
            .get(&url)
            .header(
                "Authorization",
                format!("Bearer {}", &authentication[ACCESS_TOKEN]),
            )
            .send()
            .await?;

        let post: TumblrResponse = response.json().await?;

        let caption = post
            .response
            .content
            .iter()
            .filter_map(|content| {
                if let Content::Text { text } = content {
                    Some(text.to_owned())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        let details = RecordDetails {
            title: Some(post.response.summary),
            url: post.response.post_url.parse().ok(),
            author: Some(AuthorQuery::Name(post.response.blog.name)),
            author_url: post.response.blog.url.parse().ok(),
            caption: Some(caption),
            tags: Vec::new(),
        };

        for media in post.response.content.into_iter().filter_map(|content| {
            if let Content::Image { media } = content {
                Some(media)
            } else {
                None
            }
        }) {
            if let Some(media) = media
                .into_iter()
                .max_by_key(|media| media.width * media.height)
            {
                let image_url = Url::parse(&media.url).expect("tumblr returned invalid url");

                let extension = PathBuf::from(image_url.path())
                    .extension()
                    .map(|s| format!(".{}", s.to_string_lossy().to_owned()))
                    .unwrap_or_default();

                let file_name = format!("{}{extension}", Uuid::new_v4());
                let mut writer = BufWriter::new(File::create_new(
                    chronicle.config.data_path.join(&file_name),
                )?);

                let request = reqwest::get(image_url).await?;

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

        Ok(())
    }
}
