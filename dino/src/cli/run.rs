use std::fs;

use clap::Parser;

use crate::{build_project, CmdExector, JsWorker};

#[derive(Debug, Parser)]
pub struct RunOpts {}

impl CmdExector for RunOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let filename = build_project(".")?;
        let context = fs::read_to_string(filename)?;
        let worker = JsWorker::try_new(&context)?;
        
        worker.run("await handlers.hello()")?;
        
        Ok(())
    }
}