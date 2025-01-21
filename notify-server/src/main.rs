use anyhow::Result;
use notify_server::{get_configuration, get_router};
use tokio::net::TcpListener;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt, Layer as _};

#[tokio::main]
async fn main() -> Result<()> {
    let configs = get_configuration()?;
    let console = Layer::new().pretty().with_filter(LevelFilter::DEBUG);

    tracing_subscriber::registry().with(console).init();

    let app = get_router();

    let addr = format!("{}:{}", configs.application.host, configs.application.port);
    let listener = TcpListener::bind(&addr).await?;
    info!("Listening on: {}", addr);

    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
