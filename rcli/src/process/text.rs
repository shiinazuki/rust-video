use crate::cli::TextSignFormat;
use crate::{get_reader, process_genpass};
use base64::engine::general_purpose;
use base64::Engine;
use chacha20poly1305::aead::Aead;
use chacha20poly1305::{ChaCha20Poly1305, Key, KeyInit, Nonce};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

trait TextSign {
    fn sign(&self, reader: impl Read) -> anyhow::Result<Vec<u8>>;
}

trait TextVerify {
    fn verify(&self, reader: impl Read, sig: &[u8]) -> anyhow::Result<bool>;
}

trait KeyLoader {
    fn load(path: impl AsRef<Path>) -> anyhow::Result<Self>
    where
        Self: Sized;
}

trait KeyAndNonceLoader {
    fn load_key_and_nonce(
        key_path: impl AsRef<Path>,
        nonce_path: impl AsRef<Path>,
    ) -> anyhow::Result<Self>
    where
        Self: Sized;
}

trait KeyGenerate {
    fn generate() -> anyhow::Result<Vec<Vec<u8>>>;
}

struct Blake3 {
    key: [u8; 32],
}

impl Blake3 {
    fn new(key: [u8; 32]) -> Self {
        Self { key }
    }

    fn try_new(key: &[u8]) -> anyhow::Result<Self> {
        let key = &key[..32];
        let key = key.try_into()?;
        let signer = Self::new(key);
        Ok(signer)
    }
}

impl KeyLoader for Blake3 {
    fn load(path: impl AsRef<Path>) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let key = fs::read_to_string(path)?.trim().to_string();
        Self::try_new(key.as_bytes())
    }
}
impl TextSign for Blake3 {
    fn sign(&self, mut reader: impl Read) -> anyhow::Result<Vec<u8>> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let ret = blake3::keyed_hash(&self.key, &buf);

        Ok(ret.as_bytes().to_vec())
    }
}

impl TextVerify for Blake3 {
    fn verify(&self, mut reader: impl Read, sig: &[u8]) -> anyhow::Result<bool> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;

        let hash = blake3::keyed_hash(&self.key, &buf);
        let hash = hash.as_bytes();
        Ok(hash == sig)
    }
}

impl KeyGenerate for Blake3 {
    fn generate() -> anyhow::Result<Vec<Vec<u8>>> {
        let key = process_genpass(32, true, true, true, true)?;
        let key = key.as_bytes().to_vec();
        Ok(vec![key])
    }
}

struct Ed25519Signer {
    key: SigningKey,
}

impl Ed25519Signer {
    fn new(key: SigningKey) -> Self {
        Self { key }
    }

    fn try_new(key: &[u8]) -> anyhow::Result<Self> {
        let key = SigningKey::from_bytes(key.try_into()?);
        let sign_key = Self::new(key);
        Ok(sign_key)
    }
}

impl KeyLoader for Ed25519Signer {
    fn load(path: impl AsRef<Path>) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let key = fs::read(path)?;
        Self::try_new(&key)
    }
}

impl TextSign for Ed25519Signer {
    fn sign(&self, mut reader: impl Read) -> anyhow::Result<Vec<u8>> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let sig = self.key.sign(&buf);

        Ok(sig.to_bytes().to_vec())
    }
}

impl KeyGenerate for Ed25519Signer {
    fn generate() -> anyhow::Result<Vec<Vec<u8>>> {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let pk = signing_key.verifying_key().to_bytes().to_vec();
        let signing_key = signing_key.as_bytes().to_vec();
        Ok(vec![signing_key, pk])
    }
}

struct Ed25519Verifier {
    key: VerifyingKey,
}

impl Ed25519Verifier {
    fn new(key: VerifyingKey) -> Self {
        Self { key }
    }

    fn try_new(key: &[u8]) -> anyhow::Result<Self> {
        let key = VerifyingKey::from_bytes(key.try_into()?)?;
        let verify_key = Self::new(key);
        Ok(verify_key)
    }
}

impl KeyLoader for Ed25519Verifier {
    fn load(path: impl AsRef<Path>) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let key = fs::read(path)?;
        Self::try_new(&key)
    }
}
impl TextVerify for Ed25519Verifier {
    fn verify(&self, mut reader: impl Read, sig: &[u8]) -> anyhow::Result<bool> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let sig = Signature::from_bytes(sig.try_into()?);
        let ret = self.key.verify(&buf, &sig).is_ok();
        Ok(ret)
    }
}

struct Encrypt {
    key: Key,
    nonce: Nonce,
}

impl Encrypt {
    fn new(key: Key, nonce: Nonce) -> Self {
        Self { key, nonce }
    }

    fn try_new(key: &[u8], nonce: &[u8]) -> anyhow::Result<Self> {
        let key = *Key::from_slice(key);
        let nonce = *Nonce::from_slice(nonce);
        let encrypt = Self::new(key, nonce);
        Ok(encrypt)
    }
}

impl KeyAndNonceLoader for Encrypt {
    fn load_key_and_nonce(
        key_path: impl AsRef<Path>,
        nonce_path: impl AsRef<Path>,
    ) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let key = fs::read_to_string(key_path)?.trim().to_string();
        let nonce = fs::read_to_string(nonce_path)?.trim().to_string();
        Self::try_new(key.as_bytes(), nonce.as_bytes())
    }
}

struct Decrypt {
    key: Key,
    nonce: Nonce,
}

impl Decrypt {
    fn new(key: Key, nonce: Nonce) -> Self {
        Self { key, nonce }
    }

    fn try_new(key: &[u8], nonce: &[u8]) -> anyhow::Result<Self> {
        let key = *Key::from_slice(key);
        let nonce = *Nonce::from_slice(nonce);
        let encrypt = Self::new(key, nonce);
        Ok(encrypt)
    }
}

impl KeyAndNonceLoader for Decrypt {
    fn load_key_and_nonce(
        key_path: impl AsRef<Path>,
        nonce_path: impl AsRef<Path>,
    ) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let key = fs::read_to_string(key_path)?.trim().to_string();
        let nonce = fs::read_to_string(nonce_path)?.trim().to_string();
        Self::try_new(key.as_bytes(), nonce.as_bytes())
    }
}

pub fn process_text_sign(input: &str, key: &str, format: TextSignFormat) -> anyhow::Result<String> {
    let mut reader = get_reader(input)?;
    let signed = match format {
        TextSignFormat::Blake3 => {
            let signer = Blake3::load(key)?;
            signer.sign(&mut reader)?
        }
        TextSignFormat::Ed25519 => {
            let signer = Ed25519Signer::load(key)?;
            signer.sign(&mut reader)?
        }
    };

    let signed = general_purpose::URL_SAFE_NO_PAD.encode(&signed);

    Ok(signed)
}

pub fn process_text_verify(
    input: &str,
    key: &str,
    sig: &str,
    format: TextSignFormat,
) -> anyhow::Result<bool> {
    let mut reader = get_reader(input)?;
    let sig = general_purpose::URL_SAFE_NO_PAD.decode(sig)?;

    let verify = match format {
        TextSignFormat::Blake3 => {
            let verifier = Blake3::load(key)?;

            verifier.verify(&mut reader, &sig)?
        }
        TextSignFormat::Ed25519 => {
            let verifier = Ed25519Verifier::load(key)?;
            verifier.verify(&mut reader, &sig)?
        }
    };

    Ok(verify)
}

pub fn process_text_generate(format: TextSignFormat, output: PathBuf) -> anyhow::Result<()> {
    match format {
        TextSignFormat::Blake3 => {
            let key = Blake3::generate()?;
            let name = output.join("blake3.txt");
            fs::write(name, &key[0])?
        }
        TextSignFormat::Ed25519 => {
            let key = Ed25519Signer::generate()?;
            fs::write(output.join("ed25519.sk"), &key[0])?;
            fs::write(output.join("ed25519.pk"), &key[1])?;
        }
    }
    Ok(())
}

//ChaCha20Poly1305 需要的密钥长度是 32 字节 nonce 的长度是12 字节
pub fn process_text_encrypt(input: &str, key: &str, nonce: &str) -> anyhow::Result<String> {
    let mut reader = get_reader(input)?;
    let mut buf = String::new();
    reader.read_to_string(&mut buf)?;

    let encrypt = Encrypt::load_key_and_nonce(key, nonce)?;

    let cipher = ChaCha20Poly1305::new(&encrypt.key);

    let ciphertext = cipher
        .encrypt(&encrypt.nonce, buf.as_bytes().as_ref())
        .unwrap_or(Vec::new());
    let encoded = general_purpose::STANDARD.encode(ciphertext);
    Ok(encoded)
}

pub fn process_text_decrypt(key: &str, nonce: &str, sig: &str) -> anyhow::Result<Vec<u8>> {
    let decoded = general_purpose::STANDARD.decode(sig)?;

    let decrypt = Decrypt::load_key_and_nonce(key, nonce)?;

    let cipher = ChaCha20Poly1305::new(&decrypt.key);

    let plaintext = cipher
        .decrypt(&decrypt.nonce, decoded.as_ref())
        .unwrap_or(Vec::new());

    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blake3_sign_verify() -> anyhow::Result<()> {
        let blake3 = Blake3::load("blake3.txt")?;
        let data = "hello, world!".as_bytes();
        let sig = blake3.sign(&mut &data[..])?;
        assert!(blake3.verify(&mut &data[..], &sig)?);
        Ok(())
    }

    #[test]
    fn test_ed25519_sign_verify() -> anyhow::Result<()> {
        let sk = Ed25519Signer::load("ed25519.sk")?;
        let pk = Ed25519Verifier::load("ed25519.pk")?;
        let data = "hello, world!".as_bytes();
        let sig = sk.sign(&mut &data[..])?;
        assert!(pk.verify(&mut &data[..], &sig)?);
        Ok(())
    }
}
