use clap::{ArgMatches, Parser};

use crate::{Backend, CmdExector, ReplContext, ReplDisplay};

use super::ReplResult;

#[derive(Debug, Parser)]
pub struct DescribeOpts {
    #[arg(help = "The name of the dataset")]
    pub name: String,
}

impl DescribeOpts {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

impl CmdExector for DescribeOpts {
    async fn execute<T: Backend>(self, backend: &mut T) -> anyhow::Result<()> {
        let df = backend.describe(&self.name).await?;
        df.display().await?;

        Ok(())
    }
}

// impl From<DescribeOpts> for ReplCommand {
//     fn from(value: DescribeOpts) -> Self {
//         ReplCommand::Describe(value)
//     }
// }

pub fn describe(args: ArgMatches, ctx: &mut ReplContext) -> ReplResult {
    let name = args
        .get_one::<String>("name")
        .expect("expect name")
        .to_owned();

    let cmd = DescribeOpts::new(name).into();

    ctx.send(cmd);

    Ok(None)
}
