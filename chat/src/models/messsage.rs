use std::{str::FromStr, u64};

use serde::{Deserialize, Serialize};

use super::{ChatFile, Message};
use crate::{AppError, AppState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMessage {
    pub content: String,
    pub files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListMessages {
    pub last_id: Option<u64>,
    pub limit: u64,
}

impl AppState {
    pub async fn create_message(
        &self,
        create_message: CreateMessage,
        chat_id: u64,
        user_id: u64
    ) -> Result<Message, AppError> {
        let base_dir = &self.config.base_dir;
        if create_message.content.is_empty() {
            return Err(AppError::CreateMessageError(
                "Content cannot be empty".to_string(),
            ));
        }

        for s in &create_message.files {
            let file = ChatFile::from_str(s)?;
            if !file.path(base_dir).exists() {
                return Err(AppError::CreateMessageError(format!(
                    "File {} doesn't exist",
                    s
                )));
            }
        }


        let message = sqlx::query_as(
            r#"
            INSERT INTO messages (chat_id, sender_id, content, files)
            VALUES ($1, $2, $3, $4)
            RETURNING id, chat_id, sender_id, content, files, created_at
            "#,
        )
        .bind(chat_id as i64)
        .bind(user_id as i64)
        .bind(create_message.content)
        .bind(&create_message.files)
        .fetch_one(&self.pool)
        .await?;

        Ok(message)
    }

    pub async fn list_messages(
        &self,
        list_messages: ListMessages,
        chat_id: u64,
    ) -> Result<Vec<Message>, AppError> {
        let last_id = list_messages.last_id.unwrap_or(i64::MAX as _);
        let messages = sqlx::query_as(
            r#"
            SELECT id, chat_id, sender_id, content, files, created_at
            FROM messages
            WHERE chat_id = $1
            AND id < $2
            ORDER BY id DESC
            LIMIT $3
            "#,
        )
        .bind(chat_id as i64)
        .bind(last_id as i64)
        .bind(list_messages.limit as i64)
        .fetch_all(&self.pool)
        .await?;

        Ok(messages)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[tokio::test]
    async fn create_message_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let create_message = CreateMessage {
            content: "hello".to_string(),
            files: vec![],
        };

        let message = state
            .create_message(create_message, 1, 1)
            .await
            .expect("create message failed");

        assert_eq!(message.content, "hello");

        let create_message = CreateMessage {
            content: "hello".to_string(),
            files: vec!["1".to_string()],
        };

        let message = state
            .create_message(create_message, 1, 1)
            .await
            .unwrap_err();
        assert_eq!(message.to_string(), "Invalid chat file path: 1",);

        let url = upload_dummy_file(&state)?;
        let create_message = CreateMessage {
            content: "hello".to_string(),
            files: vec![url],
        };

        let message = state
            .create_message(create_message, 1, 1)
            .await
            .expect("create message failed");

        assert_eq!(message.content, "hello");
        assert_eq!(message.files.len(), 1);

        Ok(())
    }

    #[tokio::test]
    async fn list_messages_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let list_messages = ListMessages {
            last_id: None,
            limit: 6,
        };

        let messages = state.list_messages(list_messages, 1).await?;
        println!("{:?}", messages);
        assert_eq!(messages.len(), 6);

        let last_id = messages.last().expect("last message should exists").id;

        let list_messages = ListMessages {
            last_id: Some(last_id as _),
            limit: 6,
        };

        let messages = state.list_messages(list_messages, 1).await?;
        println!("{:?}", messages);
        assert_eq!(messages.len(), 4);

        Ok(())
    }

    fn upload_dummy_file(state: &AppState) -> Result<String> {
        let file = ChatFile::new(1, "test.txt", b"hello world");
        let path = file.path(&state.config.base_dir);
        std::fs::create_dir_all(path.parent().expect("file path parent should exists"))?;
        std::fs::write(&path, b"hello world")?;

        Ok(file.url())
    }
}
