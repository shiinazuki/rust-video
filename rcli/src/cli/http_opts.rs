use crate::cli::verify_path;
use crate::{CmdExecutor, process_http_server};
use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
pub enum HttpSubCommand {
    #[command(about = "Server a directory over HTTP")]
    Server(HttpServerOpts),
}

impl CmdExecutor for HttpSubCommand {
    async fn execute(self) -> anyhow::Result<()> {
        match self {
            HttpSubCommand::Server(opts) => process_http_server(opts.path, opts.port).await?,
        };
        Ok(())
    }
}

#[derive(Debug, Parser)]
pub struct HttpServerOpts {
    #[arg(long, value_parser = verify_path, default_value = ".")]
    pub path: PathBuf,

    #[arg(long, default_value_t = 8331)]
    pub port: u16,
}
