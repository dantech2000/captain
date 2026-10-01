//! Finds Captain's tool folder and docker config on this machine.

use std::path::PathBuf;

use super::{ShellEnv, ShellEnvInput, shell_env};
use crate::cli_tools::{ToolPaths, has_plugin_dir, running_bundle};
use crate::tools::Bundle;

/// The environment for a shell whose docker CLI uses `docker_host`, with the tools
/// and config this machine has. `None` leaves `DOCKER_HOST` as Captain has it.
pub fn local_env(docker_host: Option<&str>) -> ShellEnv {
    let paths = ToolPaths::user();
    let bundle = running_bundle();
    let tool_bin = tool_bin(paths.as_ref(), bundle.as_ref());
    let docker_config = paths.as_ref().and_then(docker_config);
    let path = std::env::var_os("PATH");
    shell_env(&ShellEnvInput {
        docker_host,
        tool_bin: tool_bin.as_deref(),
        path: path.as_deref(),
        docker_config: docker_config.as_deref(),
    })
}

/// `~/.captain/bin` when its `docker` link works, else the bundle's `bin`.
fn tool_bin(paths: Option<&ToolPaths>, bundle: Option<&Bundle>) -> Option<PathBuf> {
    let linked = paths.filter(|paths| {
        let docker = paths.bin.join("docker");
        docker.is_symlink() && docker.exists()
    });
    match linked {
        Some(paths) => Some(paths.bin.clone()),
        None => bundle.map(Bundle::bin).filter(|bin| bin.is_dir()),
    }
}

/// Captain's own docker config folder, `~/.captain/docker`, which the app writes
/// when it runs the bundled docker (see `DockerCli` in captain-docker). It lists
/// the bundled Compose and Buildx first. `None` when the user's `config.json`
/// already lists Captain's plugin folder, so `docker login` keeps writing there.
fn docker_config(paths: &ToolPaths) -> Option<PathBuf> {
    if has_plugin_dir(&paths.docker_config, &paths.plugins) {
        return None;
    }
    let dir = paths.home.join(".captain/docker");
    dir.join("config.json").is_file().then_some(dir)
}
