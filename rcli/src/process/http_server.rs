use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use axum::Router;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;
use tracing::{info, warn};

#[derive(Debug)]
struct HttpServerState {
    path: PathBuf,
}

pub async fn process_http_server(path: PathBuf, port: u16) -> anyhow::Result<()> {
    let addr = format!("127.0.0.1:{}", port);
    info!("Serving {:?} on {}", path, addr);

    let state = HttpServerState { path: path.clone() };

    let app = Router::new()
        .route("/*path", get(file_handler))
        .nest_service("/tower", ServeDir::new(path))
        .with_state(Arc::new(state));

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn file_handler(
    State(state): State<Arc<HttpServerState>>,
    Path(path): Path<String>,
) -> impl IntoResponse {
    let p = std::path::Path::new(&state.path).join(path);
    info!("Reading file {:?}", p);
    if !p.exists() {
        return (
            StatusCode::NOT_FOUND,
            format!("File {} not fount", p.display()),
        )
            .into_response();
    }

    if p.is_dir() {
        let mut entries = match tokio::fs::read_dir(&p).await {
            Ok(entries) => entries,
            Err(e) => {
                warn!("Error reading directory: {:?}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to read directory".to_string(),
                )
                    .into_response();
            }
        };

        let mut file_links = String::new();

        while let Some(entry) = entries.next_entry().await.unwrap() {
            let file = entry.path();
            if file.is_file() {
                let file_name = file.file_name().unwrap().to_string_lossy();
                let file_url = format!(
                    "<a href=\"http://127.0.0.1:8331/src/{}\">{}</a><br>",
                    file_name, file_name
                );
                file_links.push_str(&file_url);
            }
        }

        // 返回包含文件链接的 HTML
        return (StatusCode::OK, Html(file_links)).into_response();
    }

    match tokio::fs::read_to_string(p).await {
        Ok(content) => {
            info!("Read {} bytes", content.len());
            (StatusCode::OK, content).into_response()
        }
        Err(e) => {
            warn!("Error reading file: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}
