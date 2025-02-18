mod configuration;
mod error;
mod handlers;
mod middlewares;
mod models;

use anyhow::Context;
use chat_core::{set_layers, verify_token, ChatDecodingKey, ChatEncodingKey, TokenVerify};
pub use chat_core::{Chat, User};
pub use configuration::{get_configuration, AppConfig};
pub use error::{AppError, ErrorOutput};
use handlers::*;
use middlewares::verify_chat;

use axum::{
    middleware::from_fn_with_state,
    routing::{get, post},
    Router,
};
// use r2d2::Pool;
// use redis::Client;
use secrecy::ExposeSecret;
use sqlx::PgPool;
use std::{fmt, ops::Deref, sync::Arc};
use tokio::fs;

#[derive(Debug, Clone)]
pub struct AppState {
    pub inner: Arc<AppStateInner>,
}

impl AppState {
    pub async fn try_new(config: AppConfig) -> Result<Self, AppError> {
        fs::create_dir_all(&config.base_dir)
            .await
            .with_context(|| "create base_dir failed")?;

        let ek = ChatEncodingKey::load(config.auth.sk.expose_secret())?;
        let dk = ChatDecodingKey::load(config.auth.pk.expose_secret())?;
        let pool = PgPool::connect(config.database.connection_string().expose_secret()).await?;

        // let redis_client =
        //     redis::Client::open(config.redis.connection_url().expose_secret().as_ref())?;
        // let redis_pool = r2d2::Pool::builder().build(redis_client)?;

        Ok(Self {
            inner: Arc::new(AppStateInner {
                config,
                ek,
                dk,
                pool,
                // redis_pool,
            }),
        })
    }
}

#[cfg(feature = "test-util")]
mod test_util {
    use super::*;
    use crate::configuration::get_configuration_test;
    use sqlx_db_tester::TestPg;

    impl AppState {
        pub async fn new_for_test() -> Result<(TestPg, Self), AppError> {
            let config = get_configuration_test().unwrap();
            let ek = ChatEncodingKey::load(&config.auth.sk.expose_secret())?;
            let dk = ChatDecodingKey::load(&config.auth.pk.expose_secret())?;

            let db_url = &config
                .database
                .connection_string()
                .expose_secret()
                .to_string();
            let (tdb, pool) = get_test_pool(Some(db_url)).await;

            // let redis_client =
            //     redis::Client::open(config.redis.connection_url().expose_secret().as_ref())?;

            // let redis_pool = r2d2::Pool::builder().build(redis_client)?;

            let state = Self {
                inner: Arc::new(AppStateInner {
                    config,
                    ek,
                    dk,
                    pool,
                    // redis_pool,
                }),
            };
            Ok((tdb, state))
        }
    }

    #[cfg(feature = "test-util")]
    pub async fn get_test_pool(db_url: Option<&str>) -> (TestPg, PgPool) {
        use sqlx::Executor;

        let db_url = match db_url {
            Some(v) => v.to_string(),
            None => "postgres://shiina:shiina%40^%40%29^%25%28%26%25@74.211.109.216:36594/chat"
                .to_string(),
        };
        let tdb = TestPg::new(db_url, std::path::Path::new("../migrations"));
        let pool = tdb.get_pool().await;

        // run prepared sql to insert test data
        let sql = include_str!("../fixtures/test.sql").split(';');
        let mut ts = pool.begin().await.expect("begin transaction failed");

        for s in sql {
            if s.trim().is_empty() {
                continue;
            }
            ts.execute(s).await.unwrap();
        }

        ts.commit().await.expect("commit transaction failed");

        (tdb, pool)
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
        &self.inner
    }
}

pub struct AppStateInner {
    pub config: AppConfig,
    pub(crate) dk: ChatDecodingKey,
    pub(crate) ek: ChatEncodingKey,
    pub(crate) pool: PgPool,
    // pub(crate) redis_pool: Pool<Client>,
}

impl fmt::Debug for AppStateInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppStateInner")
            .field("config", &self.config)
            .finish()
    }
}

pub async fn get_router(state: AppState) -> Result<Router, AppError> {
    let chat = Router::new()
        .route(
            "/{id}",
            get(get_chat_handler)
                .patch(update_chat_handler)
                .delete(delete_chat_handler)
                .post(send_message_handler),
        )
        .route("/{id}/messages", get(list_message_handler))
        .layer(from_fn_with_state(state.clone(), verify_chat))
        .route("/", get(list_chat_handler).post(create_chat_handler));

    let api = Router::new()
        .route("/users", get(list_chat_users_handler))
        .nest("/chats", chat)
        .route("/upload", post(upload_handler))
        .route("/files/{ws_id}/{*path}", get(file_handler))
        .layer(from_fn_with_state(state.clone(), verify_token::<AppState>))
        .route("/signin", post(signin_handler))
        .route("/signup", post(signup_handler));

    let app = Router::new()
        .route("/", get(index_handler))
        .nest("/api", api)
        .with_state(state);

    Ok(set_layers(app))
}
