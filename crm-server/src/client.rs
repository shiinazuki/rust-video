use anyhow::Result;
use crm_server::{
    get_configuration,
    pb::{WelcomeRequestBuilder, crm_client::CrmClient},
};
use tonic::Request;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{Layer as _, fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();
    let config = get_configuration().expect("Failed to load config");
    let addr = format!(
        "http://{}:{}",
        config.application.host, config.application.port
    );
    let mut client = CrmClient::connect(addr).await?;

    let req = WelcomeRequestBuilder::default()
        .id(Uuid::new_v4().to_string())
        .interval(97_u32)
        .content_ids([1_u32, 2, 3])
        .build()?;

    let response = client.welcome(Request::new(req)).await?.into_inner();
    info!("Response: {:#?}", response);
    Ok(())
}
