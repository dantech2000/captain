//! Changes Docker CLI contexts through the `docker` CLI, in the user's config dir:
//! `docker context create`, `update`, and `use`. Captain never removes a context.
//! See docs/features/0026-contexts-and-remote-hosts.md.

use std::path::Path;
use std::process::Command;
use std::time::Duration;

use captain_core::docker_context::read_contexts;

use crate::compose::DockerCli;
use crate::process::output_within;

/// How long a `docker context` command may take.
const TIMEOUT: Duration = Duration::from_secs(20);

/// Makes the context `name` point at `host`: `docker context create`, or `docker
/// context update` when it exists. `docker_dir` is the user's Docker config dir.
/// The error is a message for the user.
pub fn save_context(
    docker_dir: &Path,
    name: &str,
    description: &str,
    host: &str,
) -> Result<(), String> {
    let verb = match read_contexts(docker_dir).get(name) {
        Some(_) => "update",
        None => "create",
    };
    let endpoint = format!("host={host}");
    run(
        docker_dir,
        &[
            "context",
            verb,
            name,
            "--description",
            description,
            "--docker",
            &endpoint,
        ],
    )
}

/// Makes `name` the Docker CLI's default context: `docker context use`.
pub fn use_context(docker_dir: &Path, name: &str) -> Result<(), String> {
    run(docker_dir, &["context", "use", name])
}

/// Runs the `docker` CLI with `DOCKER_CONFIG` set to `docker_dir`, not to Captain's
/// own config dir, and without `DOCKER_HOST` or `DOCKER_CONTEXT`.
fn run(docker_dir: &Path, args: &[&str]) -> Result<(), String> {
    let docker = DockerCli::find().ok_or("The docker CLI is not installed.")?;
    let mut command = Command::new(&docker.binary);
    command
        .args(args)
        .env("DOCKER_CONFIG", docker_dir)
        .env_remove("DOCKER_HOST")
        .env_remove("DOCKER_CONTEXT");
    let output = output_within(command, TIMEOUT)?;
    if output.status.success() {
        tracing::info!(?args, "changed the Docker CLI contexts");
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(format!(
        "docker {} failed: {}",
        args[..2].join(" "),
        stderr.trim()
    ))
}
