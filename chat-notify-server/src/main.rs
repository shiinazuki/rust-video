use anyhow::Result;
use chat_notify_server::{get_configuration, get_router};
use tokio::net::TcpListener;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{Layer as _, fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    let configs = get_configuration()?;
    let console = Layer::new().pretty().with_filter(LevelFilter::DEBUG);

    tracing_subscriber::registry().with(console).init();

    let addr = format!("{}:{}", configs.application.host, configs.application.port);

    let app = get_router(configs).await?;

    let listener = TcpListener::bind(&addr).await?;
    info!("Listening on: {}", addr);

    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
