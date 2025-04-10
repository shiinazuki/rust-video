mod config;
mod error;
mod router;

use anyhow::Result;
use axum::{
    body::Bytes,
    extract::{Query, State},
    http::{HeaderMap, Method, Uri},
    response::{IntoResponse, Json},
    routing::{Router, any},
};
use dashmap::DashMap;
use indexmap::IndexMap;
use serde_json::json;
use std::collections::HashMap;
use tokio::net::TcpListener;
use tracing::info;

pub use config::*;
pub use error::AppError;
pub use router::*;

type ProjectRoutes = IndexMap<String, Vec<ProjectRoute>>;

#[derive(Clone)]
pub struct AppState {
    // key is hostname
    routers: DashMap<String, SwappableAppRouter>,
}

impl AppState {
    pub fn new(routers: DashMap<String, SwappableAppRouter>) -> Self {
        Self { routers }
    }
}

pub async fn start_server(port: u16, routers: DashMap<String, SwappableAppRouter>) -> Result<()> {
    let addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(addr).await?;

    info!("listening on {}", listener.local_addr()?);
    let state = AppState::new(routers);
    let app = Router::new()
        .route("/{*wildcard}", any(handler))
        .with_state(state);

    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

// we only support JSON requests and return JSON responses
#[allow(unused)]
async fn handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    method: Method,
    uri: Uri,
    Query(query): Query<serde_json::Value>,
    body: Bytes,
) -> Result<impl IntoResponse, AppError> {
    // 获取 host header
    let host = headers
        .get(axum::http::header::HOST)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    // 记录获取到的 host
    info!("host: {:?}", host);
    let host_str = host.split(':').next().unwrap_or(host); // 只保留主机名部分

    // 获取路由器
    let router = state
        .routers
        .get(host_str)
        .ok_or(AppError::HostNotFound(host_str.to_string()))?
        .load();

    // 匹配路由
    let matched = router.match_it(method, uri.path())?; // 使用请求的实际方法和路径
    let handler = matched.value;
    let params: HashMap<String, String> = matched
        .params
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();

    // 将请求体转换为 JSON
    let body: serde_json::Value =
        serde_json::from_slice(&body).map_err(|_| AppError::InvalidBody)?;

    // 返回 JSON 响应
    Ok(Json(json!({
        "handler": handler,
        "params": params,
        "query": query,
        "body": body,
    })))
}
