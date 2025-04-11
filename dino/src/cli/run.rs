use std::fs;

use clap::Parser;
use dino_server::{ProjectConfig, SwappableAppRouter, TenentRouter, start_server};
use tracing_subscriber::{
    Layer as _, filter::LevelFilter, fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt as _,
};

use crate::{CmdExector, build_project};

#[derive(Debug, Parser)]
pub struct RunOpts {
    #[arg(short, long, default_value = "3000")]
    port: u16,
}

impl CmdExector for RunOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let layer = Layer::new().with_filter(LevelFilter::INFO);
        tracing_subscriber::registry().with(layer).init();

        let filename = build_project(".")?;
        let config = filename.replace(".mjs", ".yaml");
        let code = fs::read_to_string(filename)?;
        let config = ProjectConfig::load(config)?;

        let router = SwappableAppRouter::try_new(&code, config.routes)?;
        let routers = vec![TenentRouter::new("localhost", router.clone())];

        start_server(self.port, routers).await?;

        Ok(())
    }
}
