use anyhow::Result;
use simple_redis::{Backend, network};
use tokio::net::TcpListener;
use tracing::{info, warn};

const ADDR: &str = "0.0.0.0:6379";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    info!("Simple-Redis-Server is listening on {}", ADDR);
    let listener = TcpListener::bind(ADDR).await?;

    let backend = Backend::new();
    loop {
        let (stream, raddr) = listener.accept().await?;
        info!("Accept connection from: {}", raddr);
        let cloned_backend = backend.clone();
        tokio::spawn(async move {
            match network::stream_handle(stream, cloned_backend).await {
                Ok(_) => info!("Connection from {} exited", raddr),
                Err(e) => warn!("handle error for {} : {:?}", raddr, e),
            }
        });
    }
}
