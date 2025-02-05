use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::{
    models::{CreateUser, SigninUser},
    AppError, AppState, ErrorOutput, User,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthOutput {
    token: String,
}

pub(crate) async fn signup_handler(
    State(state): State<AppState>,
    Json(create_user): Json<CreateUser>,
) -> Result<impl IntoResponse, AppError> {
    let user = User::create(&create_user, &state.pool).await?;
    let token = state.ek.sign(user)?;
    // todo 这里应该将token存储到缓存中
    let body = Json(AuthOutput { token });
    Ok((StatusCode::CREATED, body))
}

pub(crate) async fn signin_handler(
    State(state): State<AppState>,
    Json(signin_user): Json<SigninUser>,
) -> Result<impl IntoResponse, AppError> {
    let user = User::verify(&signin_user, &state.pool).await?;
    match user {
        Some(user) => {
            // todo 首先查看缓存中有没有token  有直接拿缓存的token响应 没有则重新生成token 然后存储到缓存中
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
    use crate::configuration::get_configuration_test;

    use super::*;
    use anyhow::Result;
    use http_body_util::BodyExt;

    #[tokio::test]
    async fn signup_should_work() -> Result<()> {
        let config = get_configuration_test()?;
        let create_user = CreateUser::new("iori2", "abc@ma.org", "123456");
        let (_tdb, state) = AppState::new_for_test(config).await?;
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
        let config = get_configuration_test()?;
        let create_user = CreateUser::new("iori2", "abc@ma.org", "123456");
        let (_tdb, state) = AppState::new_for_test(config).await?;
        signup_handler(State(state.clone()), Json(create_user.clone())).await?;
        let ret = signup_handler(State(state), Json(create_user))
            .await
            .into_response();
        assert_eq!(ret.status(), StatusCode::CONFLICT);
        let body = ret.into_body().collect().await?.to_bytes();

        let ret: ErrorOutput = serde_json::from_slice(&body)?;

        assert_eq!(ret.error, "email already exists: abc@ma.org");

        Ok(())
    }

    #[tokio::test]
    async fn signin_should_work() -> Result<()> {
        let config = get_configuration_test()?;
        let (_tdb, state) = AppState::new_for_test(config).await?;

        let name = "iori";
        let email = "abc@d.org";
        let password = "123456";
        let create_user = CreateUser::new(name, email, password);
        User::create(&create_user, &state.pool).await?;
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
        let config = get_configuration_test()?;
        let (_tdb, state) = AppState::new_for_test(config).await?;

        let email = "abc@d.org";
        let password = "123456";

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
