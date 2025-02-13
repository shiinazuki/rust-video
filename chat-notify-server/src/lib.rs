mod configuration;
mod sse;

pub use configuration::{get_configuration, AppConfig};

use anyhow::Result;
use axum::{
    response::{Html, IntoResponse},
    routing::{get, Router},
};
use chat_core::{Chat, Message};
use futures::StreamExt;
use secrecy::ExposeSecret;
use sqlx::postgres::PgListener;
use sse::sse_handler;
use tracing::info;

const INDEX_HTML: &str = include_str!("../index.html");

pub enum Event {
    NewChat(Chat),
    AddToChat(Chat),
    RemoveFromChat(Chat),
    NewMessage(Message),
}

pub fn get_router() -> Router {
    Router::new()
        .route("/", get(index_handler))
        .route("/events", get(sse_handler))
}

pub async fn setup_pg_listener(config: &AppConfig) -> Result<()> {
    let mut listener =
        PgListener::connect(config.database.connection_string().expose_secret()).await?;
    listener.listen("chat_updated").await?;
    listener.listen("chat_message_created").await?;

    let mut stream = listener.into_stream();

    tokio::spawn(async move {
        while let Some(Ok(notif)) = stream.next().await {
            info!("Received notification: {:?}", notif);
        }
    });
    Ok(())
}

async fn index_handler() -> impl IntoResponse {
    Html(INDEX_HTML)
}
