use std::fs;

use clap::Parser;
use dino_server::{JsWorker, Req};

use crate::{CmdExector, build_project};

#[derive(Debug, Parser)]
pub struct RunOpts {}

impl CmdExector for RunOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let filename = build_project(".")?;
        let context = fs::read_to_string(filename)?;
        let worker = JsWorker::try_new(&context)?;

        let req = Req::builder()
            .method("GET")
            .url("https://example.com")
            .build();

        let ret = worker.run("hello", req)?;
        println!("Response {:#?}", ret);

        Ok(())
    }
}
