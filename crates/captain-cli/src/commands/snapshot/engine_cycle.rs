//! Create and restore need a stopped engine: stop it, do the work, and start it
//! again if it ran.

use std::sync::Arc;

use anyhow::{Result, bail};
use captain_core::EngineHost;
use futures::executor::{block_on, block_on_stream};

/// Runs `work` with the engine stopped. If the engine was running, it starts again
/// afterwards, also when `work` failed.
pub fn with_engine_stopped<T>(
    host: &Arc<dyn EngineHost>,
    was_running: bool,
    work: impl FnOnce() -> Result<T>,
) -> Result<T> {
    if was_running {
        println!("Stopping Captain Engine.");
        block_on(host.stop())?;
    }
    let result = work();
    if was_running {
        start(host)?;
    }
    result
}

fn start(host: &Arc<dyn EngineHost>) -> Result<()> {
    for line in block_on_stream(host.start()) {
        match line {
            Ok(line) => println!("{line}"),
            Err(error) => bail!(error),
        }
    }
    Ok(())
}
