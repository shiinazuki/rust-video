use std::{env, fs, path::PathBuf};

use anyhow::Result;
use proto_builder_trait::tonic::BuilderAttributes;

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
        .with_type_attributes(&["MaterializeRequest"], &[r#"#[derive(Eq, Hash)]"#])
        .compile_protos(
            &[
                "../protos/metadata/messages.proto",
                "../protos/metadata/rpc.proto",
            ],
            &["../protos/metadata"],
        )?;

    Ok(())
}
