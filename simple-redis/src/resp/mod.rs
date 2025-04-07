mod array;
mod boolean;
mod bulk_string;
mod double;
mod integer;
mod map;
mod null;
mod resp_frame;
mod set;
mod simple_error;
mod simple_string;
mod utils;

pub use array::{RespArray, RespNullArray};
pub use bulk_string::{BulkString, NullBulkString};
pub use map::RespMap;
pub use null::RespNull;
pub use resp_frame::RespFrame;
pub use set::RespSet;
pub use simple_error::SimpleError;
pub use simple_string::SimpleString;

use bytes::BytesMut;
use enum_dispatch::enum_dispatch;
use thiserror::Error;

const BUF_CAP: usize = 4096;
const CRLF: &[u8] = b"\r\n";
const CRLF_LEN: usize = CRLF.len();

#[enum_dispatch]
pub trait RespEncode {
    fn encode(self) -> Vec<u8>;
}

impl RespEncode for () {
    fn encode(self) -> Vec<u8> {
        b"$-1\r\n".to_vec()
    }
}

pub trait RespDecode: Sized {
    const PREFIX: &'static str;
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError>;

    fn expect_length(buf: &[u8]) -> Result<usize, RespError>;
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum RespError {
    #[error("Invalid frame: {0}")]
    InvalidFrame(String),

    #[error("Invalid frame type: {0}")]
    InvalidFrameType(String),

    #[error("Invalid frame length: {0}")]
    InvalidFrameLength(isize),

    #[error("Frame is not complete")]
    NotComplete,

    #[error("Parse error: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error("From utf8 error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),

    #[error("Parse float error: {0}")]
    ParseFloatError(#[from] std::num::ParseFloatError),
}
