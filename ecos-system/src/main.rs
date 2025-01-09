use std::{fs, io::Read};

use anyhow::Context;
use ecos_system::MyError;

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
