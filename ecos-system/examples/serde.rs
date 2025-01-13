use anyhow::Result;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chacha20poly1305::{
    aead::{Aead, OsRng},
    AeadCore, ChaCha20Poly1305, KeyInit,
};
use chrono::{DateTime, Utc};
use derive_builder::Builder;
use http::Uri;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

#[serde_as]
#[derive(Debug, Builder, Serialize, Deserialize)]
#[builder(pattern = "owned")]
#[serde(rename_all = "camelCase")]
struct User {
    #[builder(setter(into))]
    name: String,

    #[builder(setter(into))]
    #[serde(rename = "privatAage")]
    age: u8,

    #[builder(setter(into))]
    data_of_birth: DateTime<Utc>,

    #[builder(setter(each(name = "skill", into)))]
    // #[serde(skip_serializing_if = "Vec::is_empty")]
    skills: Vec<String>,

    #[builder(setter(into))]
    state: WorkState,

    #[builder(setter(into))]
    #[serde(serialize_with = "b64_encode", deserialize_with = "b64_decode")]
    data: Vec<u8>,
    #[builder(setter(into))]
    #[serde(
        serialize_with = "serialize_encrypt",
        deserialize_with = "deserialize_decrypt"
    )]
    sensitive: String,

    #[serde_as(as = "Vec<DisplayFromStr>")]
    // #[builder(setter(into))]
    url: Vec<Uri>,
}

impl User {
    fn build() -> UserBuilder {
        UserBuilder::default()
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type", content = "details")]
enum WorkState {
    Working(String),

    OnLeave(DateTime<Utc>),

    Treminated,
}

const KEY: &[u8] = b"01234567890123456789012345678901";

fn main() -> Result<()> {
    let state = WorkState::OnLeave(Utc::now());

    let user = User::build()
        .name("shiina")
        .age(12)
        .data_of_birth(Utc::now())
        .skill("game")
        .skill("yellow")
        .state(state)
        .data(vec![1, 2, 3, 4, 5])
        .sensitive("sensitive")
        .url(vec!["https://example.com".parse()?])
        .build()?;

    let user_json = serde_json::to_string(&user)?;
    println!("user_json = {}", user_json);

    let user_str: User = serde_json::from_str(&user_json)?;
    println!("{:#?}", user_str.url[0].host());
    println!("{:#?}", user_str);

    Ok(())
}

fn b64_encode<S>(data: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let encoded = URL_SAFE_NO_PAD.encode(data);
    serializer.serialize_str(&encoded)
}

fn b64_decode<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let encoded = String::deserialize(deserializer)?;
    let decoded = URL_SAFE_NO_PAD
        .decode(encoded.as_bytes())
        .map_err(serde::de::Error::custom)?;
    Ok(decoded)
}

fn serialize_encrypt<S>(data: &str, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let encrypted = encrypt(data.as_bytes()).map_err(serde::ser::Error::custom)?;
    serializer.serialize_str(&encrypted)
}

fn deserialize_decrypt<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let encrypted = String::deserialize(deserializer)?;
    let decrypted = decrypt(&encrypted).map_err(serde::de::Error::custom)?;
    let decrypted = String::from_utf8(decrypted).map_err(serde::de::Error::custom)?;
    Ok(decrypted)
}

fn encrypt(data: &[u8]) -> Result<String> {
    let cipher = ChaCha20Poly1305::new(KEY.into());
    let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
    let ciphertext = cipher.encrypt(&nonce, data).unwrap();
    let nonce_cypertext: Vec<_> = nonce.iter().copied().chain(ciphertext).collect();

    let encoded = URL_SAFE_NO_PAD.encode(nonce_cypertext);
    Ok(encoded)
}

fn decrypt(encoded: &str) -> Result<Vec<u8>> {
    let decoded = URL_SAFE_NO_PAD.decode(encoded.as_bytes())?;
    let cipher = ChaCha20Poly1305::new(KEY.into());
    let nonce = decoded[..12].into();
    let decrypted = cipher.decrypt(nonce, &decoded[12..]).unwrap();
    Ok(decrypted)
}
