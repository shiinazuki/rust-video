mod connect;
mod describe;
mod head;
mod list;
mod schema;
mod sql;

use enum_dispatch::enum_dispatch;

pub use {connect::connect, describe::describe, head::head, list::list, schema::schema, sql::sql};

pub use {
    clap::Parser,
    connect::{ConnectOpts, DatasetConn},
    describe::DescribeOpts,
    head::HeadOpts,
    list::ListOpts,
    schema::SchemaOpts,
    sql::SqlOpts,
};

type ReplResult = Result<Option<String>, reedline_repl_rs::Error>;

#[derive(Debug, Parser)]
#[enum_dispatch(CmdExector)]
pub enum ReplCommand {
    #[command(
        name = "connect",
        about = "Connect to a dataset and register it to Taotie"
    )]
    Connect(ConnectOpts),

    #[command(name = "list", about = "List all registered datasets")]
    List(ListOpts),

    #[command(name = "schema", about = "Describe the schema a dataset")]
    Schema(SchemaOpts),

    #[command(name = "describe", about = "Describe a dataset")]
    Describe(DescribeOpts),
    #[command(about = "Show first rows of a dataset")]
    Head(HeadOpts),

    #[command(about = "Query a dataset using given sql")]
    Sql(SqlOpts),
}
