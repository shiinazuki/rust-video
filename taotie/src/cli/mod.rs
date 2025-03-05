mod connect;
mod describe;
mod head;
mod list;
mod sql;

pub use {connect::connect, describe::describe, head::head, list::list, sql::sql};

use clap::Parser;
use connect::ConnectOpts;
use describe::DescribeOpts;
use head::HeadOpts;
use sql::SqlOpts;

type ReplResult = Result<Option<String>, reedline_repl_rs::Error>;

#[derive(Debug, Parser)]
pub enum ReplCommand {
    #[command(
        name = "connect",
        about = "Connect to a dataset and register it to Taotie"
    )]
    Connect(ConnectOpts),

    #[command(name = "list", about = "List all registered datasets")]
    List,

    #[command(name = "describe", about = "Describe a dataset")]
    Describe(DescribeOpts),

    #[command(about = "Show first rows of a dataset")]
    Head(HeadOpts),

    #[command(about = "Query a dataset using given sql")]
    Sql(SqlOpts),
}
