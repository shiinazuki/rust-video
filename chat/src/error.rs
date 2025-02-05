use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorOutput {
    pub error: String,
}

impl ErrorOutput {
    pub fn new(error: impl Into<String>) -> Self {
        Self {
            error: error.into(),
        }
    }
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("email already exists: {0}")]
    EmailAlreadyExists(String),

    #[error("sqlx error: {0}")]
    SqlxError(#[from] sqlx::Error),

    #[error("password hash error: {0}")]
    PasswordHashError(#[from] argon2::password_hash::Error),

    #[error("encode or verify token error: {0}")]
    JsonWebTokenError(#[from] jsonwebtoken::errors::Error),

    #[error("parse pem error: {0}")]
    ChatPemError(#[from] pem::PemError),

    #[error("connection redis error: {0}")]
    RedisConnectionError(#[from] redis::RedisError),

    #[error("redis r2d2 error: {0}")]
    RedisR2d2Error(#[from] r2d2::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let status = match &self {
            AppError::EmailAlreadyExists(_) => StatusCode::CONFLICT,
            AppError::SqlxError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::PasswordHashError(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::JsonWebTokenError(_) => StatusCode::FORBIDDEN,
            AppError::ChatPemError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::RedisConnectionError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::RedisR2d2Error(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, Json(ErrorOutput::new(self.to_string()))).into_response()
    }
}
