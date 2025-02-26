mod configuration;
mod error;
mod notif;
mod sse;

use std::{ops::Deref, sync::Arc};

pub use configuration::{AppConfig, get_configuration};
pub use error::AppError;
pub use notif::{AppEvent, setup_pg_listener};

use anyhow::Result;
use axum::{
    middleware::from_fn_with_state,
    response::{Html, IntoResponse},
    routing::{Router, get},
};
use chat_core::{ChatDecodingKey, TokenVerify, User, verify_token};
use dashmap::DashMap;
use secrecy::ExposeSecret;
use sse::sse_handler;
use tokio::sync::broadcast;

pub type UserMap = Arc<DashMap<u64, broadcast::Sender<Arc<AppEvent>>>>;

const INDEX_HTML: &str = include_str!("../index.html");

#[derive(Clone)]
pub struct AppState(Arc<AppStateInner>);

pub struct AppStateInner {
    pub config: AppConfig,
    pub users: UserMap,
    dk: ChatDecodingKey,
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        let dk = ChatDecodingKey::load(config.auth.pk.expose_secret())
            .expect("Failed to load public key");
        let users = Arc::new(DashMap::new());
        Self(Arc::new(AppStateInner { config, users, dk }))
    }
}

impl TokenVerify for AppState {
    type Error = AppError;

    fn verify(&self, token: &str) -> Result<User, Self::Error> {
        Ok(self.dk.verify(token)?)
    }
}

impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub async fn get_router(config: AppConfig) -> Result<Router> {
    let state = AppState::new(config);
    setup_pg_listener(state.clone()).await?;
    let app = Router::new()
        .route("/events", get(sse_handler))
        .layer(from_fn_with_state(state.clone(), verify_token::<AppState>))
        .route("/", get(index_handler))
        .with_state(state);

    Ok(app)
}

async fn index_handler() -> impl IntoResponse {
    Html(INDEX_HTML)
}
