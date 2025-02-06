use crate::{AppError, User};
use chrono::Utc;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

const JWT_DURATION: u64 = 60 * 60 * 24 * 3;
const JWT_ISS: &str = "chat_server";
const JWT_AUD: &str = "chat_web";

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    #[serde(flatten)]
    sub: User,
    exp: u64,
    iss: String,
    aud: String,
}

pub struct ChatEncodingKey(EncodingKey);

impl ChatEncodingKey {
    pub fn load(priv_pem: &str) -> Result<Self, AppError> {
        let pem = pem::parse(priv_pem.as_bytes())?;
        let encoding_key = EncodingKey::from_ed_der(pem.contents());
        Ok(Self(encoding_key))
    }

    pub fn sign(&self, user: impl Into<User>) -> Result<String, AppError> {
        let claims = Claims {
            sub: user.into(),
            exp: Utc::now().timestamp() as u64 + JWT_DURATION,
            iss: JWT_ISS.to_string(),
            aud: JWT_AUD.to_string(),
        };
        let token = encode(
            &jsonwebtoken::Header::new(Algorithm::EdDSA),
            &claims,
            &self.0,
        )?;
        Ok(token)
    }
}

pub struct ChatDecodingKey(DecodingKey);

impl ChatDecodingKey {
    pub fn load(pub_pem: &str) -> Result<Self, AppError> {
        let pem = pem::parse(pub_pem.as_bytes())?;
        let decoding_key = DecodingKey::from_ed_der(pem.contents());
        Ok(Self(decoding_key))
    }

    pub fn verify(&self, token: &str) -> Result<User, AppError> {
        let mut validation = Validation::new(Algorithm::EdDSA);
        validation.aud = Some(HashSet::from([JWT_AUD.to_string()]));
        validation.iss = Some(HashSet::from([JWT_ISS.to_string()]));
        let claims = decode::<Claims>(token, &self.0, &validation)?;
        Ok(claims.claims.sub)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn jwt_sign_verify_should_work() -> Result<(), AppError> {
        let priv_pem = include_str!("../../ed25519.priv");
        let pub_pem = include_str!("../../ed25519.pub");

        let encoding_key = ChatEncodingKey::load(&priv_pem)?;
        let decoding_key = ChatDecodingKey::load(&pub_pem)?;

        let user = User::new(1, "shiina", "1@2.org");

        let token = encoding_key.sign(user.clone())?;
        let user2 = decoding_key.verify(&token)?;

        assert_eq!(user, user2);
        Ok(())
    }
}
