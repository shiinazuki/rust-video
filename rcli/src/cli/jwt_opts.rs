use super::verify_file;
use crate::{CmdExecutor, process_jwt_sign, process_jwt_verify};
use clap::Parser;

#[derive(Debug, Parser)]
pub enum JwtSubCommand {
    Sign(JwtSignOpts),

    Verify(JwtVerifyOpts),
}

impl CmdExecutor for JwtSubCommand {
    async fn execute(self) -> anyhow::Result<()> {
        match self {
            JwtSubCommand::Sign(opts) => opts.execute().await,
            JwtSubCommand::Verify(opts) => opts.execute().await,
        }
    }
}

#[derive(Debug, Parser)]
pub struct JwtSignOpts {
    #[arg(long, default_value = "b@b.com")]
    sub: String,

    #[arg(long, default_value = "ACME")]
    aud: String,

    #[arg(long)]
    exp: usize,

    #[arg(short, long, value_parser = verify_file)]
    key: String,
}

impl CmdExecutor for JwtSignOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let token = process_jwt_sign(&self.sub, &self.aud, self.exp, &self.key)?;
        println!("{}", token);
        Ok(())
    }
}

#[derive(Debug, Parser)]
pub struct JwtVerifyOpts {
    #[arg(short, long)]
    token: String,

    #[arg(short, long, value_parser = verify_file)]
    key: String,

    #[arg(long, default_value = "ACME")]
    aud: String,
}

impl CmdExecutor for JwtVerifyOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let user = process_jwt_verify(&self.token, &self.key, &self.aud)?;
        println!("{}", user);
        Ok(())
    }
}
