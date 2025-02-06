use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::AppError;

use super::{Chat, ChatType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateChat {
    pub name: Option<String>,
    pub members: Vec<i64>,
}

impl Chat {
    pub async fn create(
        create_chat: CreateChat,
        ws_id: u64,
        pool: &PgPool,
    ) -> Result<Self, AppError> {
        let chat = sqlx::query_as(
            r#"
            INSERT INTO chats (ws_id, name, type, members)
            VALUES ($1, $2, $3, $4)
            RETURNING id, ws_id, name, r#type, members, created_at
            "#,
        )
        .bind(ws_id as i64)
        .bind(create_chat.name)
        .bind(ChatType::Group)
        .bind(create_chat.members)
        .fetch_one(pool)
        .await?;

        Ok(chat)
    }
}
