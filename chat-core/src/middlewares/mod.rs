mod auth;
mod request_id;
mod server_time;

use std::fmt;

pub use auth::verify_token;

use axum::{middleware::from_fn, Router};
use server_time::ServerTimeLayer;
use tower::ServiceBuilder;
use tower_http::{compression::CompressionLayer, trace::TraceLayer};

use crate::User;

use self::request_id::set_request_id;

const REQUEST_ID_HEADER: &str = "x-request-id";
const SERVER_TIME_HEADER: &str = "x-server-time";

pub trait TokenVerify {
    type Error: fmt::Debug;
    fn verify(&self, token: &str) -> Result<User, Self::Error>;
}


pub fn set_layers(app: Router) -> Router {
    app.layer(
        ServiceBuilder::new()
            .layer(TraceLayer::new_for_http())
            .layer(CompressionLayer::new().gzip(true).br(true).deflate(true))
            .layer(from_fn(set_request_id))
            .layer(ServerTimeLayer),
    )
}
