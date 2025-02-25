use std::{env, fs, path::PathBuf};

use anyhow::Result;

fn main() -> Result<()> {
    let temp_dir = PathBuf::from("D:/temp");

    if !temp_dir.exists() {
        fs::create_dir_all(&temp_dir).expect("Failed to create temp directory");
    }

    unsafe {
        env::set_var("TMPDIR", &temp_dir);
        env::set_var("TEMP", &temp_dir);
        env::set_var("TMP", &temp_dir);
    }
    fs::create_dir_all("src/pb")?;
    let builder = tonic_build::configure();
    builder
        .out_dir("src/pb")
        .compile_protos(&["../protos/crm/crm.proto"], &["../protos/crm"])?;
    Ok(())
}
