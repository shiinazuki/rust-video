use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Extension, Json,
};
use hyper::StatusCode;

use crate::{models::CreateChat, AppError, AppState, Chat, User};

pub(crate) async fn list_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let chat = Chat::fetch_all(user.ws_id as _, &state.pool).await?;

    Ok((StatusCode::OK, Json(chat)))
}

pub(crate) async fn create_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Json(create_chat): Json<CreateChat>,
) -> Result<impl IntoResponse, AppError> {
    let chat = Chat::create(create_chat, user.ws_id as _, &state.pool).await?;
    Ok((StatusCode::CREATED, Json(chat)))
}

pub(crate) async fn get_chat_handler(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    let chat = Chat::get_by_id(id, &state.pool).await?;
    match chat {
        Some(chat) => Ok(Json(chat)),
        None => Err(AppError::NotFound(format!("chat id {} not found", id))),
    }
}

pub(crate) async fn update_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    Ok((StatusCode::OK, Json("anc".to_string())))
}

pub(crate) async fn delete_chat_handler() -> impl IntoResponse {
    "delete caht"
}
