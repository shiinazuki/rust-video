mod configuration;
mod error;
mod handlers;
mod middlewares;
mod models;
mod utils;

pub use configuration::{get_configuration, AppConfig};
pub use error::{AppError, ErrorOutput};
use handlers::*;
use middlewares::{set_layers, verify_token};
pub use models::User;

use axum::{
    middleware::from_fn_with_state,
    routing::{get, patch, post},
    Router,
};
use secrecy::ExposeSecret;
use sqlx::PgPool;
use std::{fmt, ops::Deref, sync::Arc};
use utils::{ChatDecodingKey, ChatEncodingKey};

#[derive(Debug, Clone)]
pub(crate) struct AppState {
    inner: Arc<AppStateInner>,
}

impl AppState {
    pub async fn try_new(config: AppConfig) -> Result<Self, AppError> {
        let ek = ChatEncodingKey::load(&config.auth.sk.expose_secret())?;
        let dk = ChatDecodingKey::load(&config.auth.pk.expose_secret())?;
        let pool = PgPool::connect(&config.database.connection_string().expose_secret()).await?;
        Ok(Self {
            inner: Arc::new(AppStateInner {
                config,
                ek,
                dk,
                pool,
            }),
        })
    }
}

#[cfg(test)]
impl AppState {
    pub async fn new_for_test(
        config: AppConfig,
    ) -> Result<(sqlx_db_tester::TestPg, Self), AppError> {
        use sqlx_db_tester::TestPg;
        let ek = ChatEncodingKey::load(&config.auth.sk.expose_secret())?;
        let dk = ChatDecodingKey::load(&config.auth.pk.expose_secret())?;

        let tdb = TestPg::new(
            config
                .database
                .connection_string()
                .expose_secret()
                .to_owned(),
            std::path::Path::new("../migrations"),
        );
        let pool = tdb.get_pool().await;
        let state = Self {
            inner: Arc::new(AppStateInner {
                config,
                ek,
                dk,
                pool,
            }),
        };
        Ok((tdb, state))
    }
}

impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub(crate) struct AppStateInner {
    pub(crate) config: AppConfig,
    pub(crate) dk: ChatDecodingKey,
    pub(crate) ek: ChatEncodingKey,
    pub(crate) pool: PgPool,
}

impl fmt::Debug for AppStateInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppStateInner")
            .field("config", &self.config)
            .finish()
    }
}

pub async fn get_router(config: AppConfig) -> Result<Router, AppError> {
    let state = AppState::try_new(config).await?;

    let api = Router::new()
        .route(
            "/chat",
            get(list_chat_handler)
                .post(create_chat_handler)
                .patch(update_chat_handler)
                .delete(delete_chat_handler),
        )
        .route(
            "/chat/{id}",
            patch(update_chat_handler)
                .delete(delete_chat_handler)
                .post(send_message_handler),
        )
        .route("/chat/{id}/messages", get(list_message_handler))
        .layer(from_fn_with_state(state.clone(), verify_token))
        .route("/signin", post(signin_handler))
        .route("/signup", post(signup_handler));

    let app = Router::new()
        .route("/", get(index_handler))
        .nest("/api", api)
        .with_state(state);

    Ok(set_layers(app))
}
