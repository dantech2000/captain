//! Finds the `docker` CLI and the Compose plugin for the Diagnostics page.

use std::process::Output;
use std::time::Duration;

use captain_core::diagnostics::ToolProbe;

use super::docker_cli::DockerCli;
use super::output::{error_message, parse_cli_version, parse_version};
use crate::process;

/// How long each command may take.
const TIMEOUT: Duration = Duration::from_secs(5);

/// The `docker` CLI and the Compose plugin, with their versions. Blocks for up to
/// a few seconds; call it from a background thread.
pub fn docker_tools() -> (ToolProbe, ToolProbe) {
    let Some(docker) = DockerCli::find() else {
        return (ToolProbe::Missing, ToolProbe::Missing);
    };
    let cli = probe(&docker, &["--version"], parse_cli_version);
    let compose = probe(
        &docker,
        &["compose", "version", "--format", "json"],
        parse_version,
    );
    let compose = match compose {
        // The CLI says "'compose' is not a docker command" without the plugin.
        ToolProbe::Broken(why) if why.contains("not a docker command") => ToolProbe::Missing,
        other => other,
    };
    (cli, compose)
}

fn probe(docker: &DockerCli, args: &[&str], parse: fn(&str) -> Option<String>) -> ToolProbe {
    match output_within(docker, args) {
        Ok(output) if output.status.success() => {
            match parse(&String::from_utf8_lossy(&output.stdout)) {
                Some(version) => ToolProbe::Found(version),
                None => ToolProbe::Broken("Captain cannot read the version.".into()),
            }
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            ToolProbe::Broken(error_message(&stderr).unwrap_or_else(|| "It failed.".into()))
        }
        Err(why) => ToolProbe::Broken(why),
    }
}

/// Runs `docker args` and waits at most [`TIMEOUT`]. A command that hangs is killed.
fn output_within(docker: &DockerCli, args: &[&str]) -> Result<Output, String> {
    let mut command = docker.command();
    command
        .args(args)
        .env_remove("DOCKER_HOST")
        .env_remove("DOCKER_CONTEXT");
    process::output_within(command, TIMEOUT)
}
