use anyhow::Result;
use chat_server::{get_configuration, get_router, AppState};
use tokio::net::TcpListener;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt, Layer as _};

#[tokio::main]
async fn main() -> Result<()> {
    let config = get_configuration()?;

    let console = Layer::new().pretty().with_filter(LevelFilter::DEBUG);

    tracing_subscriber::registry().with(console).init();

    info!("{:#?}", config);
    let addr = format!("{}:{}", config.application.host, config.application.port);

    let listener = TcpListener::bind(&addr).await?;

    info!("Listening on: {}", addr);

    let state = AppState::try_new(config).await?;
    let app = get_router(state).await?;

    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
