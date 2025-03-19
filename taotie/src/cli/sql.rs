use clap::{ArgMatches, Parser};

use crate::{Backend, CmdExector, ReplContext, ReplDisplay};

use super::ReplResult;

#[derive(Debug, Parser)]
pub struct SqlOpts {
    #[arg(help = "The SQL query")]
    pub query: String,
}

impl SqlOpts {
    pub fn new(query: String) -> Self {
        Self { query }
    }
}

impl CmdExector for SqlOpts {
    async fn execute<T: Backend>(self, backend: &mut T) -> anyhow::Result<()> {
        let df = backend.sql(&self.query).await?;
        df.display().await?;
        Ok(())
    }
}

// impl From<SqlOpts> for ReplCommand {
//     fn from(value: SqlOpts) -> Self {
//         ReplCommand::Sql(value)
//     }
// }

pub fn sql(args: ArgMatches, ctx: &mut ReplContext) -> ReplResult {
    let query = args
        .get_one::<String>("query")
        .expect("expect query")
        .to_owned();

    let cmd = SqlOpts::new(query).into();
    ctx.send(cmd);
    Ok(None)
}
