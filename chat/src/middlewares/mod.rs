mod auth;
mod request_id;
mod server_time;
mod chat;

pub use auth::verify_token;
pub use chat::verify_chat;

use axum::{middleware::from_fn, Router};
use server_time::ServerTimeLayer;
use tower::ServiceBuilder;
use tower_http::{compression::CompressionLayer, trace::TraceLayer};

use self::request_id::set_request_id;

const REQUEST_ID_HEADER: &str = "x-request-id";
const SERVER_TIME_HEADER: &str = "x-server-time";

pub fn set_layers(app: Router) -> Router {
    app.layer(
        ServiceBuilder::new()
            .layer(TraceLayer::new_for_http())
            .layer(CompressionLayer::new().gzip(true).br(true).deflate(true))
            .layer(from_fn(set_request_id))
            .layer(ServerTimeLayer),
    )
}
