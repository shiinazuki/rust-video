use std::mem;

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::{AppError, User};

use super::Workspace;

impl User {
    /// 根据 email 查询用户
    pub async fn find_by_email(email: &str, pool: &PgPool) -> Result<Option<Self>, AppError> {
        let user = sqlx::query_as(
            r#"
            SELECT id, ws_id, fullname, email, created_at FROM users WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }

    /// Create a new user
    // TODO: user transaction for workspace creation and user creation
    pub async fn create(create_user: &CreateUser, pool: &PgPool) -> Result<Self, AppError> {
        let user = Self::find_by_email(&create_user.email, pool).await?;
        if user.is_some() {
            return Err(AppError::EmailAlreadyExists(create_user.email.clone()));
        }

        // check if workspace exists, if not create one
        let ws = match Workspace::find_by_name(&create_user.woekspace, pool).await? {
            Some(ws) => ws,
            None => Workspace::create(&create_user.woekspace, 0, pool).await?,
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
        .fetch_one(pool)
        .await?;

        if ws.owner_id == 0 {
            ws.update_owner(user.id as _, pool).await?;
        }

        Ok(user)
    }

    pub async fn verify(signin_user: &SigninUser, pool: &PgPool) -> Result<Option<Self>, AppError> {
        let user: Option<User> = sqlx::query_as(
            r#"
             SELECT id, ws_id, fullname, email, password_hash, created_at
                FROM users WHERE email = $1
            "#,
        )
        .bind(&signin_user.email)
        .fetch_optional(pool)
        .await?;

        match user {
            Some(mut user) => {
                let password_hash = mem::take(&mut user.password_hash);
                let is_valid =
                    verify_passwor(&signin_user.password, &password_hash.unwrap_or_default())?;
                if is_valid {
                    Ok(Some(user))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
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
    #[serde(default = "get_default_workspace_name")]
    pub woekspace: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigninUser {
    pub email: String,
    pub password: String,
}

fn get_default_workspace_name() -> String {
    "none".to_string()
}

#[cfg(test)]
impl User {
    pub fn new(id: i64, fullname: &str, email: &str) -> Self {
        Self {
            id,
            ws_id: 0,
            fullname: fullname.to_string(),
            email: email.to_string(),
            password_hash: None,
            created_at: chrono::Utc::now(),
        }
    }
}

#[cfg(test)]
impl CreateUser {
    pub fn new(workspace: &str, fullname: &str, email: &str, password: &str) -> Self {
        Self {
            woekspace: workspace.to_string(),
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
    use crate::test_util::get_test_pool;
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
        let (_tdb, pool) = get_test_pool(None).await;

        let create_user = CreateUser::new("acme", "tom", "tom@acme.org", "123456");
        let ret = User::create(&create_user, &pool).await;
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
        let (_tdb, pool) = get_test_pool(None).await;

        let create_user = CreateUser::new("none", "shiina", "1@xxx.org", "hunted");

        let user = User::create(&create_user, &pool).await?;
        assert_eq!(user.email, create_user.email);
        assert_eq!(user.fullname, create_user.fullname);
        assert!(user.id > 0);

        let user = User::find_by_email(&create_user.email, &pool).await?;
        assert!(user.is_some());
        let user = user.unwrap();
        assert_eq!(user.email, create_user.email);
        assert_eq!(user.fullname, create_user.fullname);

        let signin_user = SigninUser::new("1@xxx.org", "hunted");
        let user = User::verify(&signin_user, &pool).await?;
        assert!(user.is_some());

        Ok(())
    }
}
