use anyhow::Result;
use dino_server::{ProjectConfig, SwappableAppRouter, TenentRouter, start_server};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{Layer as _, fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let config = include_str!("../fixtures/config.yaml");
    let config: ProjectConfig = serde_yaml::from_str(config)?;

    let code = r#"
        (function(){
                async function hello(req){
                    return {
                        status:200,
                        headers:{
                            "content-type":"application/json"
                        },
                        body: JSON.stringify(req),
                    };
                }
                return{hello:hello};
            })();
        "#;

    let routers = vec![TenentRouter::new(
        "localhost",
        SwappableAppRouter::try_new(code, config.routes)?,
    )];

    start_server(8888, routers).await?;
    Ok(())
}
