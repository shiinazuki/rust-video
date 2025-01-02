use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

trait KeyLoader {
    fn load(path: impl AsRef<Path>) -> anyhow::Result<Self>
    where
        Self: Sized;
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    aud: String,
    exp: usize,
}

struct Keys {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

impl Keys {
    fn new(encoding: EncodingKey, decoding: DecodingKey) -> Self {
        Self { encoding, decoding }
    }

    fn try_new(key: &[u8]) -> anyhow::Result<Self> {
        let encoding = EncodingKey::from_secret(key);
        let decoding = DecodingKey::from_secret(key);
        let keys = Self::new(encoding, decoding);
        Ok(keys)
    }
}

impl KeyLoader for Keys {
    fn load(path: impl AsRef<Path>) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let key = fs::read_to_string(path)?.trim().to_string();
        Self::try_new(key.as_bytes())
    }
}

pub fn process_jwt_sign(sub: &str, aud: &str, exp: usize, key: &str) -> anyhow::Result<String> {
    let exp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as usize + exp;
    let claims = Claims {
        sub: sub.to_owned(),
        aud: aud.to_string(),
        exp,
    };

    let keys = Keys::load(key)?;
    let token = encode(&Header::default(), &claims, &keys.encoding)?;

    Ok(token)
}

pub fn process_jwt_verify(token: &str, key: &str, aud: &str) -> anyhow::Result<String> {
    let keys = Keys::load(key)?;

    let mut validation = Validation::default();

    // 使用 HashSet 来设置期望的受众
    let mut audiences = HashSet::new();
    audiences.insert(aud.to_string()); // 添加预期的受众
    validation.aud = Some(audiences); // 设置受众

    let token = decode::<Claims>(token, &keys.decoding, &validation)?;
    let user_json = serde_json::to_string(&token.claims)?;
    Ok(user_json)
}
