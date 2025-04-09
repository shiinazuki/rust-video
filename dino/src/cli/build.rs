use std::env;

use clap::Parser;

use crate::{CmdExector, build_project};

#[derive(Debug, Parser)]
pub struct BuildOpts {}

impl CmdExector for BuildOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let cur_dir = env::current_dir()?.display().to_string();
        let filename = build_project(&cur_dir)?;
        eprintln!("Build success: {}", filename);

        Ok(())
    }
}
