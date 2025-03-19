use clap::{ArgMatches, Parser};

use crate::{Backend, CmdExector, ReplContext, ReplDisplay};

use super::ReplResult;

#[derive(Debug, Parser)]
pub struct SchemaOpts {
    #[arg(help = "The name of the dataset")]
    pub name: String,
}

impl SchemaOpts {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

impl CmdExector for SchemaOpts {
    async fn execute<T: Backend>(self, backend: &mut T) -> anyhow::Result<()> {
        let df = backend.schema(&self.name).await?;
        df.display().await?;

        Ok(())
    }
}

// impl From<DescribeOpts> for ReplCommand {
//     fn from(value: DescribeOpts) -> Self {
//         ReplCommand::Describe(value)
//     }
// }

pub fn schema(args: ArgMatches, ctx: &mut ReplContext) -> ReplResult {
    let name = args
        .get_one::<String>("name")
        .expect("expect name")
        .to_owned();

    let cmd = SchemaOpts::new(name).into();

    ctx.send(cmd);

    Ok(None)
}
