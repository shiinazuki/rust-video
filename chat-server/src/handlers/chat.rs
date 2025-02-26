use axum::{
    Extension, Json,
    extract::{Path, State},
    response::IntoResponse,
};
use hyper::StatusCode;

use crate::{
    AppError, AppState,
    models::{CreateChat, UpdateChat},
};
use chat_core::User;

pub(crate) async fn list_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let chat = state.fetch_chats(user.ws_id as _).await?;

    Ok((StatusCode::OK, Json(chat)))
}

pub(crate) async fn create_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Json(create_chat): Json<CreateChat>,
) -> Result<impl IntoResponse, AppError> {
    let chat = state.create_chat(create_chat, user.ws_id as _).await?;
    Ok((StatusCode::CREATED, Json(chat)))
}

pub(crate) async fn get_chat_handler(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    let chat = state.get_chat_by_id(id).await?;
    match chat {
        Some(chat) => Ok(Json(chat)),
        None => Err(AppError::NotFound(format!("chat id {} not found", id))),
    }
}

pub(crate) async fn update_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Json(update_chat): Json<UpdateChat>,
) -> Result<impl IntoResponse, AppError> {
    let chat = state
        .update_chat(update_chat, id as _, user.ws_id as _)
        .await?;
    Ok((StatusCode::OK, Json(chat)))
}

pub(crate) async fn delete_chat_handler(// State(state): State<AppState>,
    // Path(id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    // let chat = state.delete_chat_by_id(id).await?;
    // Ok((StatusCode::OK, Json(chat)))
    Ok("delete chat")
}
