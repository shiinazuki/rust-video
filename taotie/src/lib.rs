use std::{ops::Deref, process, thread};

use backend::DataFusionBackend;
use cli::{ConnectOpts, DescribeOpts, HeadOpts, ListOpts, SchemaOpts, SqlOpts};
use crossbeam_channel as mpsc;
use enum_dispatch::enum_dispatch;
use reedline_repl_rs::CallBackMap;

mod backend;
mod cli;

pub use cli::ReplCommand;
use tokio::runtime::Runtime;

pub type ReplCallBacks = CallBackMap<ReplContext, reedline_repl_rs::Error>;

#[enum_dispatch]
trait CmdExector {
    async fn execute<T: Backend>(self, backend: &mut T) -> anyhow::Result<()>;
}

trait Backend {
    type DataFrame: ReplDisplay;
    async fn connect(&mut self, opts: &ConnectOpts) -> anyhow::Result<()>;
    async fn list(&self) -> anyhow::Result<Self::DataFrame>;
    async fn schema(&self, name: &str) -> anyhow::Result<Self::DataFrame>;
    async fn describe(&self, name: &str) -> anyhow::Result<Self::DataFrame>;
    async fn head(&self, name: &str, size: usize) -> anyhow::Result<Self::DataFrame>;
    async fn sql(&self, sql: &str) -> anyhow::Result<Self::DataFrame>;
}

trait ReplDisplay {
    async fn display(self) -> anyhow::Result<()>;
}

pub struct ReplContext {
    pub tx: mpsc::Sender<ReplCommand>,
}

impl ReplContext {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded::<ReplCommand>();
        let rt = Runtime::new().expect("Failed to create runtime");

        let mut backend = DataFusionBackend::new();

        thread::Builder::new()
            .name("ReplBackend".to_string())
            .spawn(move || {
                while let Ok(cmd) = rx.recv() {
                    if let Err(e) = rt.block_on(cmd.execute(&mut backend)) {
                        eprintln!("Failed to process command: {}", e);
                    }
                }
            })
            .unwrap();
        Self { tx }
    }

    pub fn send(&self, cmd: ReplCommand) {
        if let Err(e) = self.tx.send(cmd) {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

impl Deref for ReplContext {
    type Target = mpsc::Sender<ReplCommand>;

    fn deref(&self) -> &Self::Target {
        &self.tx
    }
}

pub fn get_callbacks() -> ReplCallBacks {
    let mut callbacks = ReplCallBacks::new();
    callbacks.insert("connect".to_string(), cli::connect);
    callbacks.insert("list".to_string(), cli::list);
    callbacks.insert("schema".to_string(), cli::schema);
    callbacks.insert("describe".to_string(), cli::describe);
    callbacks.insert("head".to_string(), cli::head);
    callbacks.insert("sql".to_string(), cli::sql);

    callbacks
}
