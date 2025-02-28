use anyhow::Result;
use crm_server::{CrmService, get_configuration};
use tonic::transport::Server;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{Layer as _, fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let config = get_configuration().expect("Failed to load config");
    let addr = format!("{}:{}", config.application.host, config.application.port).parse()?;
    info!("Crm Servce ligtening on {}", addr);
    let svc = CrmService::try_new(config).await?.into_server();

    Server::builder().add_service(svc).serve(addr).await?;

    Ok(())
}
