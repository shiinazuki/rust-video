use std::sync::{Arc, Mutex};

use anyhow::Result;
use axum::{
    extract::State,
    routing::{get, patch},
    Json,
};
use chrono::{DateTime, Utc};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, Layer};

#[derive(Debug, PartialEq, Clone, Builder, Serialize, Deserialize)]
#[builder(pattern = "owned")]
struct User {
    #[builder(setter(into))]
    name: String,

    #[builder(setter(into))]
    age: u8,

    #[builder(setter(each(name = "skill", into)))]
    skills: Vec<String>,

    #[builder(setter(into))]
    dob: DateTime<Utc>,
}

impl User {
    fn build() -> UserBuilder {
        UserBuilder::default()
    }
}

#[derive(Debug, PartialEq, Clone, Builder, Serialize, Deserialize)]
#[builder(pattern = "owned")]
struct UserUpdate {
    #[builder(setter(into, strip_option))]
    age: Option<u8>,

    #[builder(setter(each(name = "skill", into), strip_option))]
    skills: Option<Vec<String>>,
}

const ADDR: &str = "127.0.0.1:8891";

#[tokio::main]
async fn main() -> Result<()> {
    let console = fmt::Layer::new().pretty().with_filter(LevelFilter::DEBUG);

    tracing_subscriber::registry().with(console).init();

    let user = User::build()
        .name("shiina")
        .age(12)
        .skill("game")
        .dob(Utc::now())
        .build()?;

    let user = Arc::new(Mutex::new(user));

    let app = axum::Router::new()
        .route("/", get(user_handler))
        .route("/", patch(update_user_handler))
        .with_state(user);
    let listener = TcpListener::bind(ADDR).await?;
    info!("Listening on {}", ADDR);
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

async fn user_handler(State(user): State<Arc<Mutex<User>>>) -> Json<User> {
    info!("req is: {:#?}", user);

    user.lock().unwrap().clone().into()
}

async fn update_user_handler(
    State(user): State<Arc<Mutex<User>>>,
    Json(user_update): Json<UserUpdate>,
) -> Json<User> {
    let mut user = user.lock().unwrap();

    if let Some(age) = user_update.age {
        user.age = age
    }
    if let Some(skills) = user_update.skills {
        user.skills = skills;
    }
    user.clone().into()
}
