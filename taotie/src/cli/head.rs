use clap::{ArgMatches, Parser};

use crate::ReplContext;

use super::{ReplCommand, ReplResult};

#[derive(Debug, Parser)]
pub struct HeadOpts {
    #[arg(long, help = "The name of the dataset")]
    pub name: String,

    #[arg(long, help = "The number of rows to show")]
    pub n: Option<usize>,
}

impl HeadOpts {
    pub fn new(name: String, n: Option<usize>) -> Self {
        Self { name, n }
    }
}

impl From<HeadOpts> for ReplCommand {
    fn from(value: HeadOpts) -> Self {
        ReplCommand::Head(value)
    }
}

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
