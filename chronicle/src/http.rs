use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::{Path, Query},
    routing::get,
    Router,
};
use oauth2::AuthorizationCode;
use serde::Deserialize;
use tokio::{
    net::TcpListener,
    sync::{watch, RwLock},
};
use tracing::error;

lazy_static::lazy_static! {
    pub static ref OAUTH2_HANDLERS: Arc<RwLock<HashMap<String, watch::Sender<AuthorizationCode>>>> = Default::default();
}

pub async fn register_oauth2_handler(
    name: impl Into<String>,
) -> RwLock<watch::Receiver<AuthorizationCode>> {
    let (tx, rx) = watch::channel(AuthorizationCode::new(String::new()));

    OAUTH2_HANDLERS.write().await.insert(name.into(), tx);

    RwLock::new(rx)
}

#[derive(Deserialize)]
struct AuthRequest {
    code: String,
}

async fn oauth_redirect(
    Query(AuthRequest { code }): Query<AuthRequest>,
    Path(service): Path<String>,
) -> String {
    let handlers = OAUTH2_HANDLERS.read().await;

    let Some(handler) = handlers.get(&service) else {
        error!("Invalid service {service}");
        return format!("invalid service: {service}");
    };

    handler
        .send(AuthorizationCode::new(code))
        .expect("could not send code");

    String::from("successfully logged in")
}

pub async fn start_http_server() {
    let app = Router::new().route("/oauth/redirect/{service}", get(oauth_redirect));

    let listener = TcpListener::bind("0.0.0.0:5001")
        .await
        .expect("could not bind to 0.0.0.0:5001");

    axum::serve(listener, app)
        .await
        .expect("could not serve http server")
}
