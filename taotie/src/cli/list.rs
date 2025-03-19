use clap::{ArgMatches, Parser};

use crate::{Backend, CmdExector, ReplContext, ReplDisplay};

use super::{ReplCommand, ReplResult};

#[derive(Debug, Parser)]
pub struct ListOpts;

pub fn list(_args: ArgMatches, ctx: &mut ReplContext) -> ReplResult {
    ctx.send(ReplCommand::List(ListOpts));
    Ok(None)
}

impl CmdExector for ListOpts {
    async fn execute<T: Backend>(self, backend: &mut T) -> anyhow::Result<()> {
        let df = backend.list().await?;
        df.display().await?;

        Ok(())
    }
}

// impl From<ListOpts> for ReplCommand {
//     fn from(value: ListOpts) -> Self {
//         ReplCommand::List(value)
//     }
// }
