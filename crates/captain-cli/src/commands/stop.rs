//! `captain stop`. It cancels a start in this process only; a start in the app
//! holds the engine lock, and the stop fails.

use anyhow::Result;
use futures::executor::block_on;

use crate::context::Context;

pub fn run(context: &Context) -> Result<()> {
    let host = context.host(&context.load_or_default());
    println!("Stopping Captain Engine.");
    block_on(host.stop())?;
    println!("Captain Engine is stopped.");
    Ok(())
}
