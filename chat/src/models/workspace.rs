use super::{ChatUser, Workspace};
use crate::AppError;

use sqlx::PgPool;

impl Workspace {
    pub async fn create(name: &str, user_id: u64, pool: &PgPool) -> Result<Self, AppError> {
        let ws = sqlx::query_as(
            r#"
            INSERT INTO workspaces (name, owner_id)   
            VALUES ($1, $2)
            RETURNING id, name, owner_id, created_at
            "#,
        )
        .bind(name)
        .bind(user_id as i64)
        .fetch_one(pool)
        .await?;

        Ok(ws)
    }

    pub async fn update_owner(&self, owner_id: u64, pool: &PgPool) -> Result<Self, AppError> {
        let ws = sqlx::query_as(
            r#"
            UPDATE workspaces SET owner_id = $1
            WHERE id = $2 and (SELECT ws_id FROM users WHERE id = $1) = $2
            RETURNING id, name, owner_id, created_at
            "#,
        )
        .bind(owner_id as i64)
        .bind(self.id)
        .fetch_one(pool)
        .await?;

        Ok(ws)
    }

    pub async fn fetch_all_chat_users(id: u64, pool: &PgPool) -> Result<Vec<ChatUser>, AppError> {
        let users = sqlx::query_as(
            r#"
            SELECT id, fullname, email 
            FROM users 
            WHERE ws_id = $1 
            ORDER BY id
            "#,
        )
        .bind(id as i64)
        .fetch_all(pool)
        .await?;
        Ok(users)
    }

    pub async fn find_by_name(name: &str, pool: &PgPool) -> Result<Option<Self>, AppError> {
        let ws = sqlx::query_as(
            r#"
            SELECT id, name, owner_id, created_at
            FROM workspaces
            WHERE name = $1
            "#,
        )
        .bind(name)
        .fetch_optional(pool)
        .await?;

        Ok(ws)
    }

    pub async fn find_by_id(id: u64, pool: &PgPool) -> Result<Option<Self>, AppError> {
        let ws = sqlx::query_as(
            r#"
            SELECT id, name, owner_id, created_at
            FROM workspaces
            WHERE id = $1
            "#,
        )
        .bind(id as i64)
        .fetch_optional(pool)
        .await?;

        Ok(ws)
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::{configuration::get_configuration_test, models::CreateUser, User};

    use super::*;
    use anyhow::Result;
    use secrecy::ExposeSecret;
    use sqlx_db_tester::TestPg;

    #[tokio::test]
    async fn workspace_should_create_and_set_owner() -> Result<()> {
        let config = get_configuration_test()?;
        let tdb = TestPg::new(
            config
                .database
                .connection_string()
                .expose_secret()
                .to_string(),
            Path::new("../migrations"),
        );
        let pool = tdb.get_pool().await;
        let ws = Workspace::create("test", 0, &pool).await?;

        let create_user = CreateUser::new(&ws.name, "shiina", "1@xxx.org", "hunted");
        let user = User::create(&create_user, &pool).await?;

        assert_eq!(ws.name, "test");

        assert_eq!(user.ws_id, ws.id);

        let ws = ws.update_owner(user.id as u64, &pool).await?;
        assert_eq!(ws.owner_id, user.id);

        Ok(())
    }

    #[tokio::test]
    async fn workspace_should_find_by_name() -> Result<()> {
        let config = get_configuration_test()?;
        let tdb = TestPg::new(
            config
                .database
                .connection_string()
                .expose_secret()
                .to_string(),
            Path::new("../migrations"),
        );
        let pool = tdb.get_pool().await;
        let _ws = Workspace::create("test", 0, &pool).await?;
        let ws = Workspace::find_by_name("test", &pool).await?;

        assert_eq!(ws.unwrap().name, "test");
        Ok(())
    }

    #[tokio::test]
    async fn workspace_should_fetch_all_chat_users() -> Result<()> {
        let config = get_configuration_test()?;
        let tdb = TestPg::new(
            config
                .database
                .connection_string()
                .expose_secret()
                .to_string(),
            Path::new("../migrations"),
        );
        let pool = tdb.get_pool().await;
        let ws = Workspace::create("test", 0, &pool).await?;

        let create_user = CreateUser::new(&ws.name, "shiina", "1@xxx.org", "hunted");
        let user1 = User::create(&create_user, &pool).await?;

        let create_user = CreateUser::new(&ws.name, "iori", "2@xxx.org", "hunted");
        let user2 = User::create(&create_user, &pool).await?;

        let users = Workspace::fetch_all_chat_users(ws.id as _, &pool).await?;

        assert_eq!(users.len(), 2);
        assert_eq!(users[0].id, user1.id);
        assert_eq!(users[1].id, user2.id);
        Ok(())
    }
}
