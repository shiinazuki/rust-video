use proto_builder_trait::tonic::BuilderAttributes;
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
        .with_derive_builder(&["WelcomeRequest", "RecallRequest", "RemindRequest"], None)
        .with_field_attributes(
            &["WelcomeRequest.content_ids"],
            &[r#"#[builder(setter(each(name="content_id", into)))]"#],
        )
        .compile_protos(
            &["../protos/crm/messages.proto", "../protos/crm/rpc.proto"],
            &["../protos/crm"],
        )?;

    Ok(())
}
