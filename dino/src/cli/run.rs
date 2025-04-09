use std::{collections::HashMap, fs};

use clap::Parser;

use crate::{build_project, CmdExector, JsWorker, Req};

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
            .headers(HashMap::new())
            .build();

        let ret = worker.run("hello", req)?;
        println!("Response {:#?}", ret);

        Ok(())
    }
}
