//! `captain docker-env`: the `DOCKER_HOST` for other tools.

use anyhow::{Context as _, Result, bail};

use crate::context::Context;

pub fn run(context: &Context) -> Result<()> {
    let settings = context.load_or_default();
    let endpoint = context
        .host(&settings)
        .endpoint()
        .context("Captain Engine is not available on this computer")?;
    println!("export DOCKER_HOST={}", quoted(&endpoint.to_string())?);
    Ok(())
}

/// `value` in single quotes for a POSIX shell, with each `'` written as `'\''`.
/// Refuses a line break, which `eval "$(captain docker-env)"` would run as a second
/// command.
fn quoted(value: &str) -> Result<String> {
    if value.contains(['\n', '\r', '\0']) {
        bail!("the engine address contains a line break: {value:?}");
    }
    Ok(format!("'{}'", value.replace('\'', r"'\''")))
}

#[cfg(test)]
mod tests;
