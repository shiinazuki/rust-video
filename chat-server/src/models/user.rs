use std::mem;

use argon2::{
    Argon2, PasswordHash, PasswordVerifier,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use serde::{Deserialize, Serialize};

use crate::{AppError, AppState};

use chat_core::{ChatUser, User};

#[allow(dead_code)]
impl AppState {
    /// 根据 email 查询用户
    pub async fn find_user_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as(
            r#"
            SELECT id, ws_id, fullname, email, created_at FROM users WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn find_user_by_id(&self, id: i64) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as(
            r#"
            SELECT id, ws_id, fullname, email, created_at
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    /// Create a new user
    // TODO: user transaction for workspace creation and user creation
    pub async fn create_user(&self, create_user: &CreateUser) -> Result<User, AppError> {
        let user = self.find_user_by_email(&create_user.email).await?;
        if user.is_some() {
            return Err(AppError::EmailAlreadyExists(create_user.email.clone()));
        }

        // check if workspace exists, if not create one
        let ws = match self.find_workspace_by_name(&create_user.workspace).await? {
            Some(ws) => ws,
            None => self.create_workspace(&create_user.workspace, 0).await?,
        };

        let password_hash = hash_password(&create_user.password)?;
        let user: User = sqlx::query_as(
            r#"
                INSERT INTO users (ws_id, email, fullname, password_hash)
                VALUES ($1, $2, $3, $4)
                RETURNING id, ws_id, fullname, email, created_at;
            "#,
        )
        .bind(ws.id)
        .bind(&create_user.email)
        .bind(&create_user.fullname)
        .bind(password_hash)
        .fetch_one(&self.pool)
        .await?;

        if ws.owner_id == 0 {
            self.update_workspace_owner(ws.id as _, user.id as _)
                .await?;
        }

        Ok(user)
    }

    pub async fn verify_user(&self, signin_user: &SigninUser) -> Result<Option<User>, AppError> {
        let user: Option<User> = sqlx::query_as(
            r#"
             SELECT id, ws_id, fullname, email, password_hash, created_at
                FROM users WHERE email = $1
            "#,
        )
        .bind(&signin_user.email)
        .fetch_optional(&self.pool)
        .await?;

        match user {
            Some(mut user) => {
                let password_hash = mem::take(&mut user.password_hash);
                let is_valid =
                    verify_passwor(&signin_user.password, &password_hash.unwrap_or_default())?;
                if is_valid { Ok(Some(user)) } else { Ok(None) }
            }
            None => Ok(None),
        }
    }

    pub async fn fetch_chat_user_by_ids(&self, ids: &[i64]) -> Result<Vec<ChatUser>, AppError> {
        let users = sqlx::query_as(
            r#"
            SELECT id, fullname, email
            FROM users
            WHERE id = ANY($1)
            "#,
        )
        .bind(ids)
        .fetch_all(&self.pool)
        .await?;

        Ok(users)
    }

    pub async fn fetch_chat_users(&self, ws_id: u64) -> Result<Vec<ChatUser>, AppError> {
        let users = sqlx::query_as(
            r#"
            SELECT id, fullname, email
            FROM users
            WHERE ws_id = $1
            "#,
        )
        .bind(ws_id as i64)
        .fetch_all(&self.pool)
        .await?;

        Ok(users)
    }
}

fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();

    Ok(password_hash)
}

fn verify_passwor(password: &str, password_hash: &str) -> Result<bool, AppError> {
    let argon2 = Argon2::default();
    let password_hash = PasswordHash::new(password_hash)?;

    let is_valid = argon2
        .verify_password(password.as_bytes(), &password_hash)
        .is_ok();

    Ok(is_valid)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUser {
    pub fullname: String,
    pub email: String,
    pub workspace: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigninUser {
    pub email: String,
    pub password: String,
}

#[cfg(test)]
impl CreateUser {
    pub fn new(workspace: &str, fullname: &str, email: &str, password: &str) -> Self {
        Self {
            workspace: workspace.to_string(),
            fullname: fullname.to_string(),
            email: email.to_string(),
            password: password.to_string(),
        }
    }
}

#[cfg(test)]
impl SigninUser {
    pub fn new(email: &str, password: &str) -> Self {
        Self {
            email: email.to_string(),
            password: password.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::AppState;
    use anyhow::Result;

    #[test]
    fn hash_password_and_verify_should_work() -> Result<()> {
        let password = "hunted";
        let password_hash = hash_password(password)?;
        assert_eq!(password_hash.len(), 97);
        assert!(verify_passwor(password, &password_hash)?);
        Ok(())
    }

    #[tokio::test]
    async fn create_duplicate_user_should_fail() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let create_user = CreateUser::new("acme", "tom", "tom@acme.org", "123456");
        let ret = state.create_user(&create_user).await;
        match ret {
            Err(AppError::EmailAlreadyExists(email)) => {
                assert_eq!(email, create_user.email);
            }
            _ => panic!("Expecting EmailAlreadyExists error"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn create_and_verify_user_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let create_user = CreateUser::new("none", "shiina", "11@xxx.org", "hunted");
        let user = state.create_user(&create_user).await?;
        assert_eq!(user.email, create_user.email);
        assert_eq!(user.fullname, create_user.fullname);
        assert!(user.id > 0);

        let user = state.find_user_by_email(&create_user.email).await?;
        assert!(user.is_some());
        let user = user.unwrap();
        assert_eq!(user.email, create_user.email);
        assert_eq!(user.fullname, create_user.fullname);

        let signin_user = SigninUser::new("11@xxx.org", "hunted");
        let user = state.verify_user(&signin_user).await?;
        assert!(user.is_some());

        Ok(())
    }

    #[tokio::test]
    async fn find_user_by_id_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;

        let user = state.find_user_by_id(1).await?;
        assert!(user.is_some());
        let user = user.unwrap();
        assert_eq!(user.id, 1);

        Ok(())
    }
}
