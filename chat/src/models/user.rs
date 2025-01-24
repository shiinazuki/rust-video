use std::mem;

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::{AppError, User};

impl User {
    /// 根据 email 查询用户
    pub async fn find_by_email(email: &str, pool: &PgPool) -> Result<Option<Self>, AppError> {
        let user = sqlx::query_as(
            r#"
            SELECT id, fullname, email, created_at FROM users WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }

    pub async fn create(create_user: &CreateUser, pool: &PgPool) -> Result<Self, AppError> {
        let password_hash = hash_password(&create_user.password)?;
        let user = Self::find_by_email(&create_user.email, &pool).await?;
        if user.is_some() {
            return Err(AppError::EmailAlreadyExists(create_user.email.clone()));
        }
        let user = sqlx::query_as(
            r#"
                INSERT INTO users ( email, fullname, password_hash)
                VALUES ($1, $2, $3)
                RETURNING id, fullname, email, created_at
            "#,
        )
        .bind(&create_user.email)
        .bind(&create_user.fullname)
        .bind(password_hash)
        .fetch_one(pool)
        .await?;

        Ok(user)
    }

    pub async fn verify(signin_user: &SigninUser, pool: &PgPool) -> Result<Option<Self>, AppError> {
        let user: Option<User> = sqlx::query_as(
            r#"
             SELECT * FROM users WHERE email = $1
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
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigninUser {
    pub email: String,
    pub password: String,
}

#[cfg(test)]
impl User {
    pub fn new(id: i64, fullname: &str, email: &str) -> Self {
        Self {
            id,
            fullname: fullname.to_string(),
            email: email.to_string(),
            password_hash: None,
            created_at: chrono::Utc::now(),
        }
    }
}

#[cfg(test)]
impl CreateUser {
    pub fn new(fullname: &str, email: &str, password: &str) -> Self {
        Self {
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

    use std::path::Path;

    use crate::configuration::get_configuration_test;

    use super::*;
    use anyhow::Result;
    use secrecy::ExposeSecret;
    use sqlx_db_tester::TestPg;

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
        let create_user = CreateUser::new("shiina", "1@xxx.org", "hunted");
        User::create(&create_user, &pool).await?;
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

        let create_user = CreateUser::new("shiina", "1@xxx.org", "hunted");

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
