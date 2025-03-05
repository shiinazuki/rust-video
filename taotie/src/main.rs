use anyhow::Result;
use reedline_repl_rs::Repl;
use taotie::{ReplCommand, ReplContext, get_callbacks};

const HISTORY_SIZE: usize = 1024;

fn main() -> Result<()> {
    let ctx = ReplContext::new();
    let callbacks = get_callbacks();
    
    let history_file =  "D://download//tmp//a.txt".parse()?;
    let mut repl = Repl::new(ctx)
        .with_history(history_file, HISTORY_SIZE)
        .with_banner("Welcome to Taotie, your dataset exploration REPL!")
        .with_derived::<ReplCommand>(callbacks);
    
    repl.run()?;

    Ok(())
}
