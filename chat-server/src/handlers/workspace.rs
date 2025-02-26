use axum::{Extension, Json, extract::State, response::IntoResponse};

use crate::{AppError, AppState};
use chat_core::User;

pub(crate) async fn list_chat_users_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let users = state.fetch_chat_users(user.ws_id as _).await?;

    Ok(Json(users))
}
