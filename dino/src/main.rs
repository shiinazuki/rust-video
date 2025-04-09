use clap::Parser;
use dino::{CmdExector, Opts};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()>{
    let opts = Opts::parse();
    opts.cmd.execute().await?;
    
    Ok(())
}
