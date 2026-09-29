//! Finds the Docker engine. The order is `DOCKER_HOST`, the current Docker CLI context,
//! then known socket paths.

use std::path::{Path, PathBuf};

use captain_core::docker_context::{config_dir, current_host};

use crate::Endpoint;
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
    /// `DOCKER_CONFIG`, the Docker CLI config dir when it is not `~/.docker`.
    pub docker_config: Option<PathBuf>,
    pub home: Option<PathBuf>,
}

impl DiscoveryInput {
    pub fn from_env() -> Self {
        let var = |name| std::env::var(name).ok().filter(|v: &String| !v.is_empty());
        Self {
            docker_host: var("DOCKER_HOST"),
            docker_context: var("DOCKER_CONTEXT"),
            docker_config: var("DOCKER_CONFIG").map(PathBuf::from),
            home: std::env::home_dir(),
        }
    }

    /// The Docker CLI config dir: `DOCKER_CONFIG`, else `~/.docker`.
    pub fn docker_dir(&self) -> Option<PathBuf> {
        config_dir(self.docker_config.clone(), self.home.as_deref())
    }

    /// The engine host of the current Docker CLI context, if it has one.
    fn context_host(&self) -> Option<String> {
        current_host(&self.docker_dir()?, self.docker_context.as_deref())
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
    Ok(Endpoint::parse(&discover_host(input, exists)?)?)
}

/// Like [`discover`], but returns the host string unparsed, so that the caller can
/// handle `ssh://` hosts, which [`Endpoint`] does not cover.
pub fn discover_host(
    input: &DiscoveryInput,
    exists: impl Fn(&Path) -> bool,
) -> Result<String, DiscoveryError> {
    if let Some(host) = input.docker_host.clone().or_else(|| input.context_host()) {
        return Ok(host);
    }
    if cfg!(windows) {
        return Ok(Endpoint::NamedPipe(WINDOWS_PIPE.into()).to_string());
    }
    socket_paths(input)
        .find(|path| exists(path))
        .map(|path| Endpoint::Unix(path).to_string())
        .ok_or(DiscoveryError::NotFound)
}

/// Where a [`Candidate`] came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CandidateSource {
    /// The `DOCKER_HOST` variable.
    DockerHost,
    /// The current Docker CLI context.
    Context,
    /// A known socket path, or the Windows named pipe.
    Socket,
}

/// An endpoint that discovery could use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    pub source: CandidateSource,
    pub endpoint: Endpoint,
}

/// Lists every endpoint that discovery could pick, in discovery order and without
/// duplicates. Hosts that do not parse are left out, and so are sockets that do not
/// exist. [`discover`] picks the first one, except that an unsupported `DOCKER_HOST`
/// is an error there.
pub fn candidates(input: &DiscoveryInput, exists: impl Fn(&Path) -> bool) -> Vec<Candidate> {
    let parsed = |host: &str, source| {
        Endpoint::parse(host)
            .ok()
            .map(|endpoint| Candidate { source, endpoint })
    };
    let mut found: Vec<Candidate> = Vec::new();
    let mut push = |candidate: Candidate| {
        if !found.iter().any(|c| c.endpoint == candidate.endpoint) {
            found.push(candidate);
        }
    };

    if let Some(candidate) = input
        .docker_host
        .as_deref()
        .and_then(|host| parsed(host, CandidateSource::DockerHost))
    {
        push(candidate);
    }
    if let Some(candidate) = input
        .context_host()
        .and_then(|host| parsed(&host, CandidateSource::Context))
    {
        push(candidate);
    }

    let socket = |endpoint| Candidate {
        source: CandidateSource::Socket,
        endpoint,
    };
    if cfg!(windows) {
        push(socket(Endpoint::NamedPipe(WINDOWS_PIPE.into())));
    } else {
        for path in socket_paths(input).filter(|path| exists(path)) {
            push(socket(Endpoint::Unix(path)));
        }
    }
    found
}

/// The known Unix socket paths, in the order discovery tries them.
fn socket_paths(input: &DiscoveryInput) -> impl Iterator<Item = PathBuf> + '_ {
    input
        .home
        .iter()
        .flat_map(|home| HOME_SOCKETS.iter().map(move |socket| home.join(socket)))
        .chain(std::iter::once(PathBuf::from(SYSTEM_SOCKET)))
}

#[cfg(test)]
mod tests;
