use concurrency::CmapMetrics;
use rand::{Rng, thread_rng};
use std::thread;
use std::time::Duration;

const N: usize = 2;
const M: usize = 4;

fn main() -> anyhow::Result<()> {
    let metrics = CmapMetrics::new();

    for idx in 0..N {
        task_worker(idx, metrics.clone())?;
    }

    for _ in 0..M {
        request_worker(metrics.clone())?;
    }

    loop {
        thread::sleep(Duration::from_secs(2));
        println!("{}", metrics);
    }
}

fn task_worker(idx: usize, metrics: CmapMetrics) -> anyhow::Result<()> {
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(thread_rng().gen_range(100..5000)));
            metrics.inc(format!("call.thread.worker.{}", idx))?;
        }
        #[allow(unreachable_code)]
        Ok::<_, anyhow::Error>(())
    });

    Ok(())
}

fn request_worker(metrics: CmapMetrics) -> anyhow::Result<()> {
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(thread_rng().gen_range(100..5000)));
            metrics.inc(format!("req.page.{}", thread_rng().gen_range(1..5)))?;
        }
        #[allow(unreachable_code)]
        Ok::<_, anyhow::Error>(())
    });
    Ok(())
}
