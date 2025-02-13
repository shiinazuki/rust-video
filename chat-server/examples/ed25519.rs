use anyhow::Result;
use std::{fmt::Debug, fs};

use pem::Pem;
use ring::signature::{Ed25519KeyPair, KeyPair};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    sub: String,
    exp: u64,
}

fn main() -> Result<()> {
    let doc = Ed25519KeyPair::generate_pkcs8(&ring::rand::SystemRandom::new())?;

    // 将私钥编码为 PEM 格式 并保存
    let pem = Pem::new("PRIVATE KEY", doc.as_ref());
    let base_path = std::env::current_dir()?;
    let priv_path = base_path.join("chat").join("ed25519.priv");
    fs::write(priv_path, pem::encode(&pem))?;

    let pair = Ed25519KeyPair::from_pkcs8(doc.as_ref())?;

    // 将公钥编码为 PEM 格式 并保存
    let pem = Pem::new("PUBLIC KEY", pair.public_key().as_ref());
    let pub_path = base_path.join("chat").join("ed25519.pub");
    fs::write(pub_path, pem::encode(&pem))?;

    // let encoding_key = EncodingKey::from_ed_der(doc.as_ref());
    // let pair = Ed25519KeyPair::from_pkcs8(doc.as_ref()).unwrap();
    // let public_key = pair.public_key().to_owned();
    // let decoding_key = DecodingKey::from_ed_der(public_key.as_ref());

    Ok(())
}
