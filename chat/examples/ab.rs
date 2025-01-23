use anyhow::{Context, Result};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use pem::Pem;
use ring::signature::{Ed25519KeyPair, KeyPair};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct User {
    id: u32,
    name: String,
    email: String,
}

impl User {
    pub fn new(id: u32, name: &str, email: &str) -> Self {
        User {
            id,
            name: name.to_string(),
            email: email.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    sub: String,
    exp: u64,
    iss: String,
    aud: String,
}

fn generate_keys() -> Result<()> {
    let doc = Ed25519KeyPair::generate_pkcs8(&ring::rand::SystemRandom::new())?;

    let pem = Pem::new("PRIVATE KEY", doc.as_ref());
    let base_path = std::env::current_dir()?;
    let priv_path = base_path.join("chat").join("ed25519.priv");
    fs::write(priv_path, pem::encode(&pem))?;

    let pair = Ed25519KeyPair::from_pkcs8(doc.as_ref())?;
    let pem = Pem::new("PUBLIC KEY", pair.public_key().as_ref());

    let pub_path = base_path.join("chat").join("ed25519.pub");
    fs::write(pub_path, pem::encode(&pem))?;

    Ok(())
}

fn main() -> Result<()> {
    // generate_keys()?; // 生成密钥，只需要运行一次

    let priv_pem = include_str!("../ed25519.priv");
    let pub_pem = include_str!("../ed25519.pub");

    let pem = pem::parse(priv_pem.as_bytes())?;
    let encoding_key = EncodingKey::from_ed_der(&pem.contents());

    let pem = pem::parse(pub_pem.as_bytes())?;
    let decoding_key = DecodingKey::from_ed_der(&pem.contents());

    let user = User::new(1, "shiina", "1@2.org");

    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let claims = Claims {
        sub: serde_json::to_string(&user)?,
        exp: now + 60 * 60 * 24 * 7, // 7 天后过期
        iss: "chat_server".to_string(),
        aud: "chat_wen".to_string(),
    };

    let token = encode(&Header::new(Algorithm::EdDSA), &claims, &encoding_key)?;
    println!("token:{:?}", token);
    let validation = Validation::new(Algorithm::EdDSA);
    let decoded = decode::<Claims>(&token, &decoding_key, &validation)?;
    let user2: User = serde_json::from_str(&decoded.claims.sub)?;

    assert_eq!(user, user2);

    println!("JWT验证成功！");

    Ok(())
}
