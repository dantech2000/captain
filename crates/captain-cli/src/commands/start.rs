//! `captain start`. It fails while another Captain process starts or stops the
//! engine; that process holds the engine lock.

use anyhow::{Result, bail};
use futures::executor::block_on_stream;

use crate::context::Context;

pub fn run(context: &Context) -> Result<()> {
    let host = context.host(&context.load_or_default());
    for line in block_on_stream(host.start()) {
        match line {
            Ok(line) => println!("{line}"),
            Err(error) => bail!(error),
        }
    }
    Ok(())
}
