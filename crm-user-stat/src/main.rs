use anyhow::Result;
use crm_user_stat::{UserStatsService, get_configuration};
use tonic::transport::Server;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{Layer as _, fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    let config = get_configuration().expect("Failed to load config");
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let addr = format!("{}:{}", config.application.host, config.application.port)
        .parse()
        .unwrap();
    info!("UserServce ligtening on {}", addr);

    let svc = UserStatsService::new(config).await.into_server();
    Server::builder().add_service(svc).serve(addr).await?;

    Ok(())
}
