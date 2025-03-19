use clap::{ArgMatches, Parser};

use crate::{Backend, CmdExector, ReplContext};

use super::ReplResult;

#[derive(Debug, Clone)]
pub enum DatasetConn {
    Postgres(String),
    Csv(String),
    Parquet(String),
    NdJson(String),
}

#[derive(Debug, Parser)]
pub struct ConnectOpts {
    #[arg(value_parser = verify_conn_str,
        help = "Connection string to the dataset, cloud be postgres or local file (support: csv, parquet, json)")]
    pub conn: DatasetConn,

    #[arg(short, long, help = "If database, the name of the table")]
    pub table: Option<String>,

    #[arg(short, long, help = "The name of the dataset")]
    pub name: String,
}

impl ConnectOpts {
    pub fn new(conn: DatasetConn, table: Option<String>, name: String) -> Self {
        Self { conn, table, name }
    }
}

impl CmdExector for ConnectOpts {
    async fn execute<T: Backend>(self, backend: &mut T) -> anyhow::Result<()> {
        backend.connect(&self).await?;
        println!("Connected to dataset: {}", self.name);
        Ok(())
    }
}

// impl From<ConnectOpts> for ReplCommand {
//     fn from(value: ConnectOpts) -> Self {
//         ReplCommand::Connect(value)
//     }
// }

pub fn connect(args: ArgMatches, ctx: &mut ReplContext) -> ReplResult {
    let conn = args
        .get_one::<DatasetConn>("conn")
        .expect("expect conn")
        .to_owned();
    let table = args.get_one::<String>("table").map(|s| s.to_string());
    let name = args
        .get_one::<String>("name")
        .expect("expect name")
        .to_owned();

    let cmd = ConnectOpts::new(conn, table, name).into();

    ctx.send(cmd);

    Ok(None)
}

fn verify_conn_str(s: &str) -> Result<DatasetConn, String> {
    let conn_str = s.to_string();
    if conn_str.starts_with("postgres://") {
        Ok(DatasetConn::Postgres(conn_str))
    } else if conn_str.ends_with(".csv") {
        Ok(DatasetConn::Csv(conn_str))
    } else if conn_str.ends_with(".parquet") {
        Ok(DatasetConn::Parquet(conn_str))
    } else if conn_str.ends_with(".ndjson") {
        Ok(DatasetConn::NdJson(conn_str))
    } else {
        Err(format!("Invalid connection string: {}", s))
    }
}
