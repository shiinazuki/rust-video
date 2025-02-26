use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::{
    io,
    net::{TcpListener, TcpStream},
};
use tracing::{info, level_filters::LevelFilter, warn};
use tracing_subscriber::{Layer as _, fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    listen_addr: String,
    upstream_addr: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let console = Layer::new().pretty().with_filter(LevelFilter::DEBUG);
    tracing_subscriber::registry().with(console).init();

    let config = Arc::new(resolve_config());

    info!("Upstream: {}", config.upstream_addr);
    info!("Listen: {}", config.listen_addr);

    let listener = TcpListener::bind(&config.listen_addr).await?;

    loop {
        let (client, addr) = listener.accept().await?;
        let cloned_config = Arc::clone(&config);
        info!("Accept connection from: {}", addr);
        tokio::spawn(async move {
            let upstream = TcpStream::connect(&cloned_config.upstream_addr).await?;

            proxy(client, upstream).await?;

            Ok::<(), anyhow::Error>(())
        });
    }
}

async fn proxy(mut client: TcpStream, mut upstream: TcpStream) -> Result<()> {
    let (mut client_reader, mut client_writer) = client.split();
    let (mut upstream_reader, mut upstream_writer) = upstream.split();

    // 将数据从 client 复制到上游服务器
    let client_to_upstream = io::copy(&mut client_reader, &mut upstream_writer);
    // 将数据从上游服务器复制到 client
    let upstream_to_client = io::copy(&mut upstream_reader, &mut client_writer);

    // 并发的执行上面两个 future
    if let Err(e) = tokio::try_join!(client_to_upstream, upstream_to_client) {
        warn!("Error: {}", e);
    }

    Ok(())
}

fn resolve_config() -> Config {
    Config {
        upstream_addr: "127.0.0.1:8891".to_string(),
        listen_addr: "127.0.0.1:8892".to_string(),
    }
}
