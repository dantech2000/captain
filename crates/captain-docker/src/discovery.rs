//! Finds the Docker engine. The order is `DOCKER_HOST`, the current Docker CLI context,
//! then known socket paths.

use std::path::{Path, PathBuf};

use crate::Endpoint;
use crate::docker_context::current_context_host;
use crate::endpoint::UnsupportedHost;

/// Unix sockets to try, relative to the home directory, in order.
const HOME_SOCKETS: &[&str] = &[
    ".docker/run/docker.sock",     // Docker Desktop
    ".orbstack/run/docker.sock",   // OrbStack
    ".colima/default/docker.sock", // Colima
    ".colima/docker.sock",         // Colima (older)
    ".rd/docker.sock",             // Rancher Desktop
];
const SYSTEM_SOCKET: &str = "/var/run/docker.sock";
const WINDOWS_PIPE: &str = "//./pipe/docker_engine";

/// The environment that discovery reads. Tests build it by hand.
#[derive(Debug, Clone, Default)]
pub struct DiscoveryInput {
    pub docker_host: Option<String>,
    pub docker_context: Option<String>,
    pub home: Option<PathBuf>,
}

impl DiscoveryInput {
    pub fn from_env() -> Self {
        let var = |name| std::env::var(name).ok().filter(|v: &String| !v.is_empty());
        Self {
            docker_host: var("DOCKER_HOST"),
            docker_context: var("DOCKER_CONTEXT"),
            home: std::env::home_dir(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DiscoveryError {
    #[error(transparent)]
    Unsupported(#[from] UnsupportedHost),
    #[error("no Docker engine found; start one or set DOCKER_HOST")]
    NotFound,
}

/// Picks the endpoint to connect to. `exists` checks whether a socket path exists.
pub fn discover(
    input: &DiscoveryInput,
    exists: impl Fn(&Path) -> bool,
) -> Result<Endpoint, DiscoveryError> {
    if let Some(host) = &input.docker_host {
        return Ok(Endpoint::parse(host)?);
    }

    let docker_dir = input.home.as_ref().map(|home| home.join(".docker"));
    let context_host = docker_dir
        .as_deref()
        .and_then(|dir| current_context_host(dir, input.docker_context.as_deref()));
    if let Some(host) = context_host {
        return Ok(Endpoint::parse(&host)?);
    }

    if cfg!(windows) {
        return Ok(Endpoint::NamedPipe(WINDOWS_PIPE.into()));
    }

    let home_sockets = input
        .home
        .iter()
        .flat_map(|home| HOME_SOCKETS.iter().map(move |socket| home.join(socket)));
    home_sockets
        .chain(std::iter::once(PathBuf::from(SYSTEM_SOCKET)))
        .find(|path| exists(path))
        .map(Endpoint::Unix)
        .ok_or(DiscoveryError::NotFound)
}

#[cfg(test)]
mod tests;
