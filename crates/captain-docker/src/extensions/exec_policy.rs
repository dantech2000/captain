//! Keeps `docker.cli.exec` and `vm.cli.exec` on Captain's engine. The page picks the
//! command, arguments, and environment, so it must not be able to point the `docker`
//! CLI at another daemon with a global option or a `DOCKER_*` variable.

use std::collections::BTreeMap;
use std::process::Command;

use captain_core::EngineError;
use captain_core::extension::{ExecRequest, ExecScope};

/// `DOCKER_*` variables a page may still set. None of them picks the daemon.
const ALLOWED_DOCKER_ENV: [&str; 3] = [
    "DOCKER_BUILDKIT",
    "DOCKER_CLI_HINTS",
    "DOCKER_DEFAULT_PLATFORM",
];

/// Refuses an exec that could choose the daemon. Host binaries are the extension's
/// own programs, so only their environment gets fixed, in [`point_at`].
pub(super) fn check(scope: ExecScope, exec: &ExecRequest) -> Result<(), EngineError> {
    if scope == ExecScope::Host {
        return Ok(());
    }
    // The CLI reads global options such as `--host` and `--context` only before the
    // subcommand, so the subcommand itself must not be an option.
    if exec.cmd.is_empty() || exec.cmd.starts_with('-') {
        return Err(EngineError::Api(format!(
            "\"{}\" is not a docker command",
            exec.cmd
        )));
    }
    if let Some(name) = exec.env.keys().find(|name| picks_daemon(name)) {
        return Err(EngineError::Api(format!("extensions cannot set {name}")));
    }
    Ok(())
}

/// Applies the page's `env`, then points `command` at `host`, so the page's values
/// cannot win.
pub(super) fn point_at(command: &mut Command, host: &str, env: &BTreeMap<String, String>) {
    command
        .envs(env)
        .env("DOCKER_HOST", host)
        // DOCKER_HOST and DOCKER_CONTEXT together make the CLI refuse to run.
        .env_remove("DOCKER_CONTEXT");
}

fn picks_daemon(name: &str) -> bool {
    let name = name.to_ascii_uppercase();
    name.starts_with("DOCKER_") && !ALLOWED_DOCKER_ENV.contains(&name.as_str())
}

#[cfg(test)]
mod tests;
