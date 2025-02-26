use std::{fs, io::Read};

use anyhow::Context;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(#[from] std::num::ParseIntError),

    #[error("Custom error: {0}")]
    Custom(String),

    #[error("Serialize json error: {0}")]
    Serialize(#[from] serde_json::Error),
}

fn main() -> Result<(), anyhow::Error> {
    println!("size of MyError is {}", std::mem::size_of::<MyError>());
    let filename = "D:/settingsa.json";
    let mut fd =
        fs::File::open(filename).with_context(|| format!("Can not find file: {}", filename))?;
    let mut str = String::new();
    fd.read_to_string(&mut str)?;
    println!("{:#?}", str);

    fail_with_error()?;
    Ok(())
}

fn fail_with_error() -> Result<(), MyError> {
    Err(MyError::Custom("This is a custom error".to_string()))
}
