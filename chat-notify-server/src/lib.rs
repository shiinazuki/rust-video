mod configuration;
mod error;
mod notif;
mod sse;

use std::{ops::Deref, sync::Arc};

pub use configuration::{get_configuration, AppConfig};
pub use error::AppError;
pub use notif::{setup_pg_listener, AppEvent};

use anyhow::Result;
use axum::{
    middleware::from_fn_with_state,
    response::{Html, IntoResponse},
    routing::{get, Router},
};
use chat_core::{verify_token, ChatDecodingKey, TokenVerify, User};
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
        let dk = ChatDecodingKey::load(&config.auth.pk.expose_secret())
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

pub fn get_router(config: AppConfig) -> (Router, AppState) {
    let state = AppState::new(config);
    let app = Router::new()
        .route("/events", get(sse_handler))
        .layer(from_fn_with_state(state.clone(), verify_token::<AppState>))
        .route("/", get(index_handler))
        .with_state(state.clone());
    (app, state)
}

async fn index_handler() -> impl IntoResponse {
    Html(INDEX_HTML)
}
