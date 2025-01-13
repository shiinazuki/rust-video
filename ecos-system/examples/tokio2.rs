// 将异步运行时将数据传给同步的thread
use std::{thread, time::Duration};

use anyhow::Result;
use tokio::sync::mpsc::{self, Receiver};

#[tokio::main]
async fn main() -> Result<()> {
    let (tx, rx) = mpsc::channel(32);
    let handle = worker(rx);

    tokio::spawn(async move {
        let mut i = 0;
        loop {
            i += 1;
            println!("Send task: {}!", i);
            tx.send(format!("task {}", i)).await?;
        }
        #[allow(unreachable_code)]
        Ok::<(), anyhow::Error>(())
    });

    handle.join().unwrap();
    Ok(())
}

fn worker(mut rx: Receiver<String>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        while let Some(s) = rx.blocking_recv() {
            let ret = expensive_blocking_task(s);
            println!("result: {}", ret);
        }
    })
}

fn expensive_blocking_task(s: String) -> String {
    std::thread::sleep(Duration::from_millis(800));
    blake3::hash(s.as_bytes()).to_string()
}
