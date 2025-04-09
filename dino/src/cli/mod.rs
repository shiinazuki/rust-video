mod build;
mod init;
mod run;

use clap::Parser;
use enum_dispatch::enum_dispatch;
pub use {build::BuildOpts, init::InitOpts, run::RunOpts};

#[derive(Debug, Parser)]
#[command(name="dino", version, author, about, long_about = None)]
pub struct Opts {
    #[command(subcommand)]
    pub cmd: SubCommand,
}

#[derive(Debug, Parser)]
#[enum_dispatch(CmdExector)]
pub enum SubCommand {
    #[command(name = "init", about = "init dino project")]
    Init(InitOpts),

    #[command(name = "build", about = "build dino project")]
    Build(BuildOpts),

    #[command(name = "run", about = "Run user's dino project")]
    Run(RunOpts),
}
