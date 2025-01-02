use bytes::BytesMut;
use enum_dispatch::enum_dispatch;
use std::collections::BTreeMap;
use std::ops::{Deref, DerefMut};

#[enum_dispatch]
pub trait RespEncode {
    fn encode(self) -> Vec<u8>;
}

pub trait RespDecode {
    fn decode(buf: Self) -> anyhow::Result<RespFrame, String>;
}

impl RespDecode for BytesMut {
    fn decode(_buf: Self) -> anyhow::Result<RespFrame, String> {
        Ok(RespFrame::Boolean(false))
    }
}

#[enum_dispatch(RespEncode)]
#[derive(Debug, PartialOrd, PartialEq)]
pub enum RespFrame {
    Integer(i64),
    SimpleString(SimpleString),
    Error(SimpleError),
    BulkString(BulkString),
    NullBulkString(RespNullBulkString),
    Array(RespArray),
    NullArray(RespNullArray),
    Null(RespNull),
    Boolean(bool),
    Double(f64),
    Map(RespMap),
    Set(RespSet),
}

#[derive(Debug, PartialOrd, PartialEq)]
pub struct SimpleString(pub String);

impl SimpleString {
    pub fn new(s: impl Into<String>) -> Self {
        SimpleString(s.into())
    }
}

impl Deref for SimpleString {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, PartialOrd, PartialEq)]
pub struct SimpleError(pub String);

impl SimpleError {
    pub fn new(s: impl Into<String>) -> Self {
        SimpleError(s.into())
    }
}

impl Deref for SimpleError {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, PartialOrd, PartialEq)]
pub struct BulkString(pub Vec<u8>);

impl BulkString {
    pub fn new(s: impl Into<Vec<u8>>) -> Self {
        BulkString(s.into())
    }
}

impl Deref for BulkString {
    type Target = Vec<u8>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, PartialOrd, PartialEq)]
pub struct RespNull;

#[derive(Debug, PartialOrd, PartialEq)]
pub struct RespArray(pub Vec<RespFrame>);

impl RespArray {
    pub fn new(s: impl Into<Vec<RespFrame>>) -> Self {
        RespArray(s.into())
    }
}

impl Deref for RespArray {
    type Target = Vec<RespFrame>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, PartialOrd, PartialEq)]
pub struct RespNullArray;

#[derive(Debug, PartialOrd, PartialEq)]
pub struct RespNullBulkString;

#[derive(Debug, PartialOrd, PartialEq)]
pub struct RespMap(pub BTreeMap<String, RespFrame>);

impl RespMap {
    pub fn new() -> Self {
        RespMap(BTreeMap::new())
    }
}

impl Default for RespMap {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for RespMap {
    type Target = BTreeMap<String, RespFrame>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RespMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, PartialOrd, PartialEq)]
pub struct RespSet(pub Vec<RespFrame>);

impl RespSet {
    pub fn new(s: impl Into<Vec<RespFrame>>) -> Self {
        RespSet(s.into())
    }
}

impl Deref for RespSet {
    type Target = Vec<RespFrame>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
