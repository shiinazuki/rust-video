use clap::{ArgMatches, Parser};

use crate::ReplContext;

use super::{ReplCommand, ReplResult};

#[derive(Debug, Parser)]
pub struct DescribeOpts {
    #[arg(short, long, help = "The name of the dataset")]
    pub name: String,
}

impl DescribeOpts {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

impl From<DescribeOpts> for ReplCommand {
    fn from(value: DescribeOpts) -> Self {
        ReplCommand::Describe(value)
    }
}

pub fn describe(args: ArgMatches, ctx: &mut ReplContext) -> ReplResult {
    let name = args
        .get_one::<String>("name")
        .expect("expect name")
        .to_owned();

    let cmd = DescribeOpts::new(name).into();

    ctx.send(cmd);

    Ok(None)
}
