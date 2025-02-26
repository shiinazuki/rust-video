use crate::cli::Base64Format;
use crate::get_reader;
use base64::Engine;
use base64::engine::general_purpose;
use std::io::Read;

pub fn process_encode_base64(input: &str, format: Base64Format) -> anyhow::Result<String> {
    let mut reader = get_reader(input)?;
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;

    let encoded = match format {
        Base64Format::Standard => general_purpose::STANDARD.encode(&buf),
        Base64Format::UrlSafe => general_purpose::URL_SAFE_NO_PAD.encode(&buf),
    };

    Ok(encoded)
}

pub fn process_decode_base64(input: &str, format: Base64Format) -> anyhow::Result<Vec<u8>> {
    let mut reader = get_reader(input)?;

    let mut buf = String::new();
    reader.read_to_string(&mut buf)?;

    let buf = buf.trim();
    let decoded = match format {
        Base64Format::Standard => general_purpose::STANDARD.decode(buf)?,
        Base64Format::UrlSafe => general_purpose::URL_SAFE_NO_PAD.decode(buf)?,
    };

    Ok(decoded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_encode() {
        let input = "D:\\soft\\rust\\project\\items\\rust-video\\rcli\\Cargo.toml";
        let format = Base64Format::Standard;
        assert!(process_encode_base64(input, format).is_ok());
    }

    #[test]
    fn test_process_decode() {
        let input = "D:\\soft\\rust\\project\\items\\rust-video\\rcli\\b64.txt";
        let format = Base64Format::Standard;
        assert!(process_decode_base64(input, format).is_ok());
    }
}
