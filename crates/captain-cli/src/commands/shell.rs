//! `captain shell [-- cmd...]`: `limactl shell captain` with the terminal attached.

use anyhow::{Result, bail};
use captain_host::LimaHost;
use futures::executor::block_on;

use crate::context::Context;

pub fn run(context: &Context, command: &[String]) -> Result<()> {
    if !cfg!(target_os = "macos") {
        bail!("captain shell needs Captain Engine's VM, which runs on macOS only.");
    }
    let host: LimaHost = context.lima_host(&context.load_or_default());
    let status = block_on(captain_core::EngineHost::status(&host))?;
    if !status.is_running() {
        bail!(
            "Captain Engine is {}. Run `captain start` first.",
            status.label().to_lowercase()
        );
    }
    exec(host.shell(command)?)
}

/// Replaces this process, so the exit code and signals pass through.
#[cfg(unix)]
fn exec(mut command: std::process::Command) -> Result<()> {
    use std::os::unix::process::CommandExt;
    Err(command.exec().into())
}

#[cfg(not(unix))]
fn exec(mut command: std::process::Command) -> Result<()> {
    let status = command.status()?;
    std::process::exit(status.code().unwrap_or(1));
}
