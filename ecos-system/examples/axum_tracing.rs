use std::time::Duration;

use axum::{
    extract::Request,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};

use tokio::{
    net::TcpListener,
    time::{sleep, Instant},
};
use tracing::{debug, info, instrument, level_filters::LevelFilter, warn};
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    Layer,
};

const ADDR: &str = "0.0.0.0:8012";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let file_appender = tracing_appender::rolling::daily("ecos-system/tmp/logs", "scosystem.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let console = fmt::Layer::new()
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .pretty()
        .with_filter(LevelFilter::DEBUG);

    let file = fmt::Layer::new()
        .pretty()
        .with_writer(non_blocking)
        .with_filter(LevelFilter::INFO);

    tracing_subscriber::registry()
        .with(console)
        .with(file)
        .init();

    let app = Router::new().route("/", get(index_handler));

    info!("Starting server on {}", ADDR);
    let listener = TcpListener::bind(ADDR).await?;

    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

#[instrument(fields(http.uri = req.uri().path(), http.method = req.method().as_str()))]
async fn index_handler(req: Request) -> Response {
    debug!("index handler started");
    sleep(Duration::from_millis(11)).await;
    let ret = long_task().await;
    info!(http.status = 200, "index handler complteed");
    ret
}

#[instrument]
async fn long_task() -> Response {
    let start = Instant::now();
    sleep(Duration::from_millis(112)).await;
    let elapsed = start.elapsed().as_millis() as u64;
    warn!(app.task_duration = elapsed, "task takes too long");
    "Hello, Rust!".into_response()
}
