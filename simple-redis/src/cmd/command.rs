use crate::{Backend, RespArray, RespError, RespFrame, SimpleString};
use enum_dispatch::enum_dispatch;
use lazy_static::lazy_static;
use thiserror::Error;

lazy_static! {
    pub static ref RESP_OK: RespFrame = SimpleString::new("OK").into();
}

#[enum_dispatch]
pub trait CommandExecutor {
    fn execute(self, backend: &Backend) -> RespFrame;
}

#[enum_dispatch(CommandExecutor)]
#[derive(Debug)]
pub enum Command {
    Get(GetCommand),
    Set(SetCommand),
    HGet(HGetCommand),
    HSet(HSetCommand),
    HGetAll(HGetAllCommand),
    Unrecognized(UnrecognizedCommand),
}

impl TryFrom<RespFrame> for Command {
    type Error = CommandError;

    fn try_from(value: RespFrame) -> Result<Self, Self::Error> {
        match value {
            RespFrame::Array(array) => array.try_into(),
            _ => Err(CommandError::InvalidCommand(
                "Command must be an Array".to_string(),
            )),
        }
    }
}

impl TryFrom<RespArray> for Command {
    type Error = CommandError;
    fn try_from(v: RespArray) -> Result<Self, Self::Error> {
        match v.first() {
            Some(RespFrame::BulkString(ref cmd)) => match cmd.as_ref() {
                b"get" => Ok(GetCommand::try_from(v)?.into()),
                b"set" => Ok(SetCommand::try_from(v)?.into()),
                b"hget" => Ok(HGetCommand::try_from(v)?.into()),
                b"hset" => Ok(HSetCommand::try_from(v)?.into()),
                b"hgetall" => Ok(HGetAllCommand::try_from(v)?.into()),
                _ => Ok(UnrecognizedCommand.into()),
            },
            _ => Err(CommandError::InvalidCommand(
                "Command must have a BulkString as the first argument".to_string(),
            )),
        }
    }
}

#[derive(Error, Debug)]
pub enum CommandError {
    #[error("Invalid command: {0}")]
    InvalidCommand(String),

    #[error("Invalid arguments: {0}")]
    InvalidArgument(String),

    #[error("{0}")]
    RespError(#[from] RespError),

    #[error("From utf8 error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
}

#[derive(Debug)]
pub struct GetCommand {
    pub(crate) key: String,
}

#[derive(Debug)]
pub struct SetCommand {
    pub(crate) key: String,
    pub(crate) value: RespFrame,
}

#[derive(Debug)]
pub struct HGetCommand {
    pub(crate) key: String,
    pub(crate) field: String,
}

#[derive(Debug)]
pub struct HSetCommand {
    pub(crate) key: String,
    pub(crate) field: String,
    pub(crate) value: RespFrame,
}

#[derive(Debug)]
pub struct HGetAllCommand {
    pub(crate) key: String,
}

#[derive(Debug)]
pub struct UnrecognizedCommand;

impl CommandExecutor for UnrecognizedCommand {
    fn execute(self, _backend: &Backend) -> RespFrame {
        RESP_OK.clone()
    }
}

pub fn validate_command(
    value: &RespArray,
    names: &[&'static str],
    n_args: usize,
) -> Result<(), CommandError> {
    if value.len() != n_args + names.len() {
        return Err(CommandError::InvalidArgument(format!(
            "{} command must have exactly {} argument",
            names.join(" "),
            n_args
        )));
    }

    for (i, name) in names.iter().enumerate() {
        match value[i] {
            RespFrame::BulkString(ref cmd) => {
                if cmd.to_ascii_lowercase() != name.as_bytes() {
                    return Err(CommandError::InvalidCommand(format!(
                        "Invalid command: expected {}, got {}",
                        name,
                        String::from_utf8_lossy(cmd.as_ref())
                    )));
                }
            }
            _ => {
                return Err(CommandError::InvalidCommand(
                    "GET command must have a BulkString as the first argument".to_string(),
                ));
            }
        }
    }
    Ok(())
}

pub fn extract_args(value: RespArray, start: usize) -> Result<Vec<RespFrame>, CommandError> {
    Ok(value.0.into_iter().skip(start).collect::<Vec<RespFrame>>())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RespDecode, RespNull};
    use anyhow::Result;
    use bytes::BytesMut;

    #[test]
    fn test_command() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*2\r\n$3\r\nget\r\n$5\r\nhello\r\n");

        let frame = RespArray::decode(&mut buf)?;

        let cmd: Command = frame.try_into()?;

        let backend = Backend::new();

        let ret = cmd.execute(&backend);
        assert_eq!(ret, RespFrame::Null(RespNull));

        Ok(())
    }
}
