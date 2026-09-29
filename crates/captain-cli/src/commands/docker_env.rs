//! `captain docker-env`: the `DOCKER_HOST` for other tools.

use anyhow::{Context as _, Result};

use crate::context::Context;

pub fn run(context: &Context) -> Result<()> {
    let settings = context.load_or_default();
    let endpoint = context
        .host(&settings)
        .endpoint()
        .context("Captain Engine is not available on this computer")?;
    println!("export DOCKER_HOST='{endpoint}'");
    Ok(())
}
