//! The `docker` binary Captain runs, with the config folder that makes it use the
//! bundled Compose and Buildx plugins.

use std::path::{Path, PathBuf};
use std::process::Command;

use captain_core::tools::{Bundle, docker_config};

use super::locate::locate_docker;

/// A `docker` binary and, for a bundled one, Captain's `DOCKER_CONFIG` folder.
#[derive(Debug, Clone)]
pub struct DockerCli {
    pub binary: PathBuf,
    config_dir: Option<PathBuf>,
}

impl DockerCli {
    /// The bundled `docker`, or the first one on `PATH` or in a known folder. With
    /// bundled plugins, this also writes `~/.captain/docker/config.json`.
    pub fn find() -> Option<Self> {
        let bundle = std::env::current_exe()
            .ok()
            .and_then(|exe| Bundle::from_exe(&exe));
        let path = std::env::var_os("PATH");
        let home = std::env::home_dir();
        let binary = locate_docker(
            bundle.as_ref(),
            path.as_deref(),
            home.as_deref(),
            Path::is_file,
        )?;
        let plugins = bundle
            .map(|bundle| bundle.cli_plugins())
            .filter(|dir| dir.is_dir());
        let config_dir = match (plugins, home) {
            (Some(plugins), Some(home)) => write_config(&plugins, &home),
            _ => None,
        };
        Some(Self { binary, config_dir })
    }

    /// A command for this binary, with `DOCKER_CONFIG` set when Captain has its own.
    pub fn command(&self) -> Command {
        let mut command = Command::new(&self.binary);
        if let Some(dir) = &self.config_dir {
            command.env("DOCKER_CONFIG", dir);
        }
        command
    }
}

/// Writes `~/.captain/docker/config.json` from the user's config and returns the
/// folder. The user's `~/.docker` stays untouched. `None`, with a warning, if the
/// file cannot be written; the CLI then uses the user's config and plugins.
fn write_config(plugins: &Path, home: &Path) -> Option<PathBuf> {
    let user_dir = std::env::var_os("DOCKER_CONFIG")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".docker"));
    let user = std::fs::read_to_string(user_dir.join("config.json")).ok();
    let text = docker_config(user.as_deref(), plugins, &user_dir.join("cli-plugins"));
    let dir = home.join(".captain/docker");
    match write_private(&dir.join("config.json"), &text) {
        Ok(()) => Some(dir),
        Err(error) => {
            tracing::warn!(%error, dir = %dir.display(), "cannot write the docker config");
            None
        }
    }
}

/// Writes `text` to `file`, readable only by the user: it can hold registry
/// credentials copied from the user's config.
fn write_private(file: &Path, text: &str) -> std::io::Result<()> {
    use std::io::Write;
    if let Some(dir) = file.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    options.open(file)?.write_all(text.as_bytes())
}
