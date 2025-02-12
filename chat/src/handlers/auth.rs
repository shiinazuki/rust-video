use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
// use redis::Commands;
use serde::{Deserialize, Serialize};

use crate::{
    models::{CreateUser, SigninUser},
    AppError, AppState, ErrorOutput,
};

// const REDIS_EX_TIME: u64 = 60 * 60 * 24 * 3;

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthOutput {
    token: String,
}

pub(crate) async fn signup_handler(
    State(state): State<AppState>,
    Json(create_user): Json<CreateUser>,
) -> Result<impl IntoResponse, AppError> {
    let user = state.create_user(&create_user).await?;

    // let redis_key = user.id.clone();

    let token = state.ek.sign(user)?;

    // 将token存储到缓存中
    // let mut conn = state.redis_pool.get()?;
    // let _: Result<(), redis::RedisError> = conn.set_ex(redis_key, token.clone(), REDIS_EX_TIME);

    let body = Json(AuthOutput { token });
    Ok((StatusCode::CREATED, body))
}

pub(crate) async fn signin_handler(
    State(state): State<AppState>,
    Json(signin_user): Json<SigninUser>,
) -> Result<impl IntoResponse, AppError> {
    let user = state.verify_user(&signin_user).await?;
    match user {
        Some(user) => {
            // 查看缓存中有没有token  有直接拿缓存的token响应 没有则重新生成token 然后存储到缓存中
            // let redis_key = user.id.clone();
            // let mut conn = state.redis_pool.get()?;
            // let token: Result<String, redis::RedisError> = conn.get(redis_key);
            // let token = match token {
            //     Ok(v) => v,
            //     Err(_) => {
            //         let token = state.ek.sign(user)?;
            //         let _: Result<(), redis::RedisError> =
            //             conn.set_ex(redis_key, token.clone(), REDIS_EX_TIME);
            //         token
            //     }
            // };

            let token = state.ek.sign(user)?;
            Ok((StatusCode::OK, Json(AuthOutput { token })).into_response())
        }
        None => {
            let body = Json(ErrorOutput::new("Invalid email or password"));

            Ok((StatusCode::FORBIDDEN, body).into_response())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use http_body_util::BodyExt;

    #[tokio::test]
    async fn signup_should_work() -> Result<()> {
        let create_user = CreateUser::new("foo", "iori2", "abc@ma.org", "123456");
        let (_tdb, state) = AppState::new_for_test().await?;
        let ret = signup_handler(State(state), Json(create_user))
            .await?
            .into_response();
        assert_eq!(ret.status(), StatusCode::CREATED);
        let body = ret.into_body().collect().await?.to_bytes();
        let ret: AuthOutput = serde_json::from_slice(&body)?;
        println!("{:?}", ret);
        Ok(())
    }

    #[tokio::test]
    async fn signup_duplicate_user_should_409() -> Result<()> {
        let create_user = CreateUser::new("foo", "test", "test@acme.org", "123456");
        let (_tdb, state) = AppState::new_for_test().await?;
        let ret = signup_handler(State(state), Json(create_user))
            .await
            .into_response();
        assert_eq!(ret.status(), StatusCode::CONFLICT);
        let body = ret.into_body().collect().await?.to_bytes();

        let ret: ErrorOutput = serde_json::from_slice(&body)?;

        assert_eq!(ret.error, "email already exists: test@acme.org");

        Ok(())
    }

    #[tokio::test]
    async fn signin_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;

        let email = "tom@acme.org";
        let password = "123456";
        let signin_user = SigninUser::new(email, password);

        let ret = signin_handler(State(state), Json(signin_user))
            .await?
            .into_response();
        assert_eq!(ret.status(), StatusCode::OK);
        let body = ret.into_body().collect().await?.to_bytes();
        let ret: AuthOutput = serde_json::from_slice(&body)?;
        println!("{:?}", ret);
        Ok(())
    }

    #[tokio::test]
    async fn signin_with_non_exist_user_should_403() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;

        let email = "tom@acme.org";
        let password = "1234567";

        let signin_user = SigninUser::new(email, password);

        let ret = signin_handler(State(state), Json(signin_user))
            .await
            .into_response();
        assert_eq!(ret.status(), StatusCode::FORBIDDEN);
        let body = ret.into_body().collect().await?.to_bytes();
        let ret: ErrorOutput = serde_json::from_slice(&body)?;
        assert_eq!(ret.error, "Invalid email or password");
        Ok(())
    }
}
