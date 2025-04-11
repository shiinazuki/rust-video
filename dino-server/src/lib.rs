mod config;
mod engine;
mod error;
mod router;

use anyhow::Result;
use axum::{
    body::Bytes,
    extract::{Query, State},
    http::{HeaderMap, Method, Uri},
    response::{IntoResponse, Response},
    routing::{Router, any},
};
use dashmap::DashMap;
use indexmap::IndexMap;
use matchit::Match;
use std::collections::HashMap;
use tokio::net::TcpListener;
use tracing::info;

pub use config::*;
pub use engine::*;
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

#[derive(Clone)]
pub struct TenentRouter {
    host: String,
    router: SwappableAppRouter,
}

impl TenentRouter {
    pub fn new(host: impl Into<String>, router: SwappableAppRouter) -> Self {
        Self {
            host: host.into(),
            router,
        }
    }
}

pub async fn start_server(port: u16, routers: Vec<TenentRouter>) -> Result<()> {
    let addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(addr).await?;

    info!("listening on {}", listener.local_addr()?);
    let map = DashMap::new();
    for TenentRouter { host, router } in routers {
        map.insert(host, router);
    }
    let state = AppState::new(map);
    let app = Router::new()
        .route("/{*wildcard}", any(handler))
        .with_state(state);

    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

#[allow(unused)]
async fn handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    method: Method,
    uri: Uri,
    Query(query): Query<HashMap<String, String>>,
    body: Bytes,
) -> Result<impl IntoResponse, AppError> {
    let router = get_router_by_host(&headers, state)?;
    let uri_clone = uri.clone();
    let matched = router.match_it(method.clone(), uri_clone.path())?;
    let req = assemble_req(&matched, headers, body, method, uri, query)?;
    let handler = matched.value;
    let worker = JsWorker::try_new(&router.code)?;
    let res = worker.run(handler, req)?;

    Ok(Response::from(res))
}

fn get_router_by_host(headers: &HeaderMap, state: AppState) -> Result<AppRouter, AppError> {
    let host = headers
        .get(axum::http::header::HOST)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    info!("host: {:?}", host);
    let host_str = host.split(':').next().unwrap_or(host);

    let router = state
        .routers
        .get(host_str)
        .ok_or(AppError::HostNotFound(host_str.to_string()))?
        .load();

    Ok(router)
}

fn assemble_req(
    matched: &Match<&str>,
    headers: HeaderMap,
    body: Bytes,
    method: Method,
    uri: Uri,
    query: HashMap<String, String>,
) -> Result<Req, AppError> {
    let params: HashMap<String, String> = matched
        .params
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();

    let headers = headers
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap().to_string()))
        .collect();

    let body = String::from_utf8(body.into()).ok();

    let req = Req::builder()
        .method(method.to_string())
        .url(uri.to_string())
        .query(query)
        .params(params)
        .headers(headers)
        .body(body.clone())
        .build();

    Ok(req)
}
