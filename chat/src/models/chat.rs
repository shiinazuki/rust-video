use super::{Chat, ChatType};
use crate::{AppError, AppState};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CreateChat {
    pub name: Option<String>,
    pub members: Vec<i64>,
    pub public: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateChat {
    pub name: Option<String>,
    pub members: Vec<i64>,
    pub public: bool,
}

impl AppState {
    pub async fn create_chat(&self, create_chat: CreateChat, ws_id: u64) -> Result<Chat, AppError> {
        let len = create_chat.members.len();
        if len < 2 {
            return Err(AppError::CreateChatError(
                "Chat must have at least 2 members".to_string(),
            ));
        }

        if len > 8 && create_chat.name.is_none() {
            return Err(AppError::CreateChatError(
                "Group chat with more than 8 members must have a name".to_string(),
            ));
        }

        // verify if all members exist
        let users = self.fetch_chat_user_by_ids(&create_chat.members).await?;
        if users.len() != len {
            return Err(AppError::CreateChatError(
                "Some members do not exist".to_string(),
            ));
        }

        let chat_type = match (&create_chat.name, len) {
            (None, 2) => ChatType::Single,
            (None, _) => ChatType::Group,
            (Some(_), _) => {
                if create_chat.public {
                    ChatType::PublicChannel
                } else {
                    ChatType::PrivateChannel
                }
            }
        };

        let chat = sqlx::query_as(
            r#"
            INSERT INTO chats (ws_id, name, type, members)
            VALUES ($1, $2, $3, $4)
            RETURNING id, ws_id, name, type, members, created_at
            "#,
        )
        .bind(ws_id as i64)
        .bind(create_chat.name)
        .bind(chat_type)
        .bind(create_chat.members)
        .fetch_one(&self.pool)
        .await?;

        Ok(chat)
    }

    pub async fn fetch_chats(&self, ws_id: u64) -> Result<Vec<Chat>, AppError> {
        let chats = sqlx::query_as(
            r#"
            SELECT id, ws_id, name, type, members, created_at
            From chats
            Where ws_id = $1
            "#,
        )
        .bind(ws_id as i64)
        .fetch_all(&self.pool)
        .await?;

        Ok(chats)
    }

    pub async fn get_chat_by_id(&self, id: u64) -> Result<Option<Chat>, AppError> {
        let chat = sqlx::query_as(
            r#"
            SELECT id, ws_id, name, type, members, created_at
            FROM chats
            WHERE id = $1
            "#,
        )
        .bind(id as i64)
        .fetch_optional(&self.pool)
        .await?;

        Ok(chat)
    }

    pub async fn update_chat(
        &self,
        update_chat: UpdateChat,
        id: u64,
        ws_id: u64,
    ) -> Result<Chat, AppError> {
        let len = update_chat.members.len();

        let chat_type = match (&update_chat.name, len) {
            (None, 2) => ChatType::Single,
            (None, _) => ChatType::Group,
            (Some(_), _) => {
                if update_chat.public {
                    ChatType::PublicChannel
                } else {
                    ChatType::PrivateChannel
                }
            }
        };

        let chat = sqlx::query_as(
            r#"
        UPDATE chats SET name = $1, type = $2, members = $3, ws_id = $4
        WHERE id = $5
        RETURNING id, ws_id, name, type, members, created_at
        "#,
        )
        .bind(update_chat.name)
        .bind(chat_type)
        .bind(update_chat.members)
        .bind(ws_id as i64)
        .bind(id as i64)
        .fetch_one(&self.pool)
        .await?;

        Ok(chat)
    }

    // pub async fn delete_chat_by_id(&self, id: u64) -> Result<Chat, AppError> {
    //     let chat = sqlx::query_as(
    //         r#"
    //         DELETE FROM chats
    //         WHERE id = $1
    //         RETURNING id, ws_id, name, type, members, created_at
    //         "#,
    //     )
    //     .bind(id as i64)
    //     .fetch_one(&self.pool)
    //     .await?;

    //     Ok(chat)
    // }

    pub async fn is_chat_member(&self, chat_id: u64, user_id: u64) -> Result<bool, AppError> {
        let is_member = sqlx::query(
            r#"
            SELECT 1
            FROM chats
            WHERE id = $1 AND $2 = ANY(members)

            "#,
        )
        .bind(chat_id as i64)
        .bind(user_id as i64)
        .fetch_optional(&self.pool)
        .await?;

        Ok(is_member.is_some())
    }
}

#[cfg(test)]
impl CreateChat {
    pub fn new(name: &str, members: &[i64], public: bool) -> Self {
        let name = if name.is_empty() {
            None
        } else {
            Some(name.to_string())
        };
        Self {
            name,
            members: members.to_vec(),
            public,
        }
    }
}

#[cfg(test)]
impl UpdateChat {
    pub fn new(name: &str, members: &[i64], public: bool) -> Self {
        let name = if name.is_empty() {
            None
        } else {
            Some(name.to_string())
        };
        Self {
            name,
            members: members.to_vec(),
            public,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Context, Result};

    #[tokio::test]
    async fn create_single_chat_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let create_chat = CreateChat::new("", &[1, 2], false);
        let chat = state
            .create_chat(create_chat, 1)
            .await
            .with_context(|| "create chat failed")?;

        assert_eq!(chat.ws_id, 1);
        assert_eq!(chat.members.len(), 2);
        assert_eq!(chat.r#type, ChatType::Single);

        Ok(())
    }

    #[tokio::test]
    async fn create_public_named_chat_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let create_chat = CreateChat::new("general", &[1, 2, 3], true);
        let chat = state
            .create_chat(create_chat, 1)
            .await
            .with_context(|| "create chat failed")?;

        assert_eq!(chat.ws_id, 1);
        assert_eq!(chat.members.len(), 3);
        assert_eq!(chat.r#type, ChatType::PublicChannel);

        Ok(())
    }

    #[tokio::test]
    async fn chat_get_by_id_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let chat = state
            .get_chat_by_id(1)
            .await
            .with_context(|| "get chat by id failed")?
            .unwrap();

        assert_eq!(chat.id, 1);
        assert_eq!(chat.name.unwrap(), "general");
        assert_eq!(chat.ws_id, 1);
        assert_eq!(chat.members.len(), 5);

        Ok(())
    }

    #[tokio::test]
    async fn chat_fetch_all_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let chats = state
            .fetch_chats(1)
            .await
            .with_context(|| "fetch all chats failed")?;

        assert_eq!(chats.len(), 4);

        Ok(())
    }

    #[tokio::test]
    async fn chat_is_member_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let is_member = state.is_chat_member(1, 1).await.expect("is member failed");
        assert!(is_member);

        let is_member = state.is_chat_member(1, 6).await.expect("is member failed");
        assert!(!is_member);

        let is_member = state.is_chat_member(10, 1).await.expect("is member failed");
        assert!(!is_member);

        let is_member = state.is_chat_member(2, 4).await.expect("is member failed");
        assert!(!is_member);
        Ok(())
    }

    #[tokio::test]
    async fn update_chat_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let update_chat = UpdateChat::new("update_general", &[1, 2, 3], false);
        let chat = state
            .update_chat(update_chat, 1, 2)
            .await
            .with_context(|| "update chat failed")?;

        assert_eq!(chat.ws_id, 2);
        assert_eq!(chat.name.unwrap(), "update_general");
        assert_eq!(chat.members.len(), 3);
        assert_eq!(chat.r#type, ChatType::PrivateChannel);

        Ok(())
    }

    // #[tokio::test]
    // async fn delete_by_id_chat_should_work() -> Result<()> {
    //     let (_tdb, state) = AppState::new_for_test().await?;
    //     let chat = state
    //         .delete_chat_by_id(1)
    //         .await
    //         .with_context(|| "delete chat failed")?;

    //     assert_eq!(chat.ws_id, 1);
    //     assert_eq!(chat.name.unwrap(), "general");
    //     assert_eq!(chat.members.len(), 5);
    //     assert_eq!(chat.r#type, ChatType::PublicChannel);

    //     Ok(())
    // }
}
