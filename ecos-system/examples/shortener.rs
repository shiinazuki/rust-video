use std::sync::Arc;

use anyhow::Result;
use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
    routing::{get, post},
};
use ecos_system::{ApplicationSettings, get_configuration};
use http::{HeaderMap, StatusCode, header::LOCATION};
use nanoid::nanoid;
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use tokio::net::TcpListener;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{Layer as _, fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, Serialize, Deserialize)]
struct ShortenReq {
    url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ShortenResp {
    url: String,
}

#[derive(Debug)]
struct AppState {
    db: PgPool,
    application: ApplicationSettings,
}

impl AppState {
    async fn try_new(url: &str, application: ApplicationSettings) -> Result<Self> {
        let pool = PgPool::connect(url).await?;
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS urls(
            id CHAR(6) PRIMARY KEY,
            url TEXT NOT NULL UNIQUE
            )
            "#,
        )
        .execute(&pool)
        .await?;
        Ok(Self {
            db: pool,
            application,
        })
    }

    async fn shorten(&self, url: &str) -> Result<String> {
        let id = nanoid!(6);
        let ret: UrlRecord = sqlx::query_as(
            "INSERT INTO urls (id, url) VALUES ($1, $2) ON CONFLICT(url) DO UPDATE SET url=EXCLUDED.url RETURNING id",
        )
        .bind(&id)
        .bind(url)
        .fetch_one(&self.db)
        .await?;
        Ok(ret.id)
    }

    async fn get_url(&self, id: &str) -> Result<String> {
        let ret: UrlRecord = sqlx::query_as("SELECT url FROM urls WHERE id = $1")
            .bind(id)
            .fetch_one(&self.db)
            .await?;
        Ok(ret.url)
    }
}

#[derive(Debug, FromRow)]
struct UrlRecord {
    #[sqlx(default)]
    id: String,
    #[sqlx(default)]
    url: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let configuration = get_configuration()?;

    let console = Layer::new().pretty().with_filter(LevelFilter::DEBUG);

    tracing_subscriber::registry().with(console).init();

    let addr = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    info!("Listening on: {}", addr);
    let listener = TcpListener::bind(addr).await?;

    let url = configuration.database.connection_string();
    let state = Arc::new(AppState::try_new(&url.expose_secret(), configuration.application).await?);

    let app = axum::Router::new()
        .route("/", post(shorten_handle))
        .route("/{id}", get(redirect_handle))
        .with_state(state);

    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

// body 的 读取转换需要放在最后
async fn shorten_handle(
    State(state): State<Arc<AppState>>,
    Json(data): Json<ShortenReq>,
) -> Result<impl IntoResponse, StatusCode> {
    let id = state
        .shorten(&data.url)
        .await
        .map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?;

    let body = Json(ShortenResp {
        url: format!(
            "http://{}:{}/{}",
            state.application.host, state.application.port, id
        ),
    });
    Ok((StatusCode::CREATED, body))
}

async fn redirect_handle(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, StatusCode> {
    let url = state
        .get_url(&id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    let mut headers = HeaderMap::new();
    headers.insert(LOCATION, url.parse().expect("Failed parse url"));

    Ok((StatusCode::PERMANENT_REDIRECT, headers))
}
