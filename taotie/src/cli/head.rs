use clap::{ArgMatches, Parser};

use crate::{Backend, CmdExector, ReplContext, ReplDisplay};

use super::ReplResult;

#[derive(Debug, Parser)]
pub struct HeadOpts {
    #[arg(help = "The name of the dataset")]
    pub name: String,

    #[arg(long, help = "The number of rows to show")]
    pub n: Option<usize>,
}

impl HeadOpts {
    pub fn new(name: String, n: Option<usize>) -> Self {
        Self { name, n }
    }
}

impl CmdExector for HeadOpts {
    async fn execute<T: Backend>(self, backend: &mut T) -> anyhow::Result<()> {
        let df = backend.head(&self.name, self.n.unwrap_or(5)).await?;
        df.display().await?;
        Ok(())
    }
}

// impl From<HeadOpts> for ReplCommand {
//     fn from(value: HeadOpts) -> Self {
//         ReplCommand::Head(value)
//     }
// }

pub fn head(args: ArgMatches, ctx: &mut ReplContext) -> ReplResult {
    let name = args
        .get_one::<String>("name")
        .expect("expect name")
        .to_owned();
    let n = args.get_one::<usize>("n").copied();

    let cmd = HeadOpts::new(name, n).into();

    ctx.send(cmd);

    Ok(None)
}
