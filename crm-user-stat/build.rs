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
        .with_serde(
            &["User"],
            true,
            true,
            Some(&[r#"#[serde(rename_all = "camelCase")]"#]),
        )
        .with_derive_builder(
            &[
                "User",
                "QueryRequest",
                "RawQueryRequest",
                "TimeQuery",
                "IdQuery",
            ],
            None,
        )
        .with_field_attributes(
            &["User.email", "User.name", "RawQueryRequest.query"],
            &[r#"#[builder(setter(into))]"#],
        )
        .with_sqlx_from_row(&["User"], None)
        .with_field_attributes(
            &["TimeQuery.brfore", "TimeQuery.after"],
            &[r#"#[builder(setter(into, strip_option))]"#],
        )
        .with_field_attributes(
            &["QueryRequest.timestamps"],
            &[r#"#[builder(setter(each(name="timestamp", into)))]"#],
        )
        .with_field_attributes(
            &["QueryRequest.ids"],
            &[r#"#[builder(setter(each(name="id", into)))]"#],
        )
        .compile_protos(
            &[
                "../protos/user-stats/messages.proto",
                "../protos/user-stats/rpc.proto",
            ],
            &["../protos/user-stats"],
        )?;

    Ok(())
}
