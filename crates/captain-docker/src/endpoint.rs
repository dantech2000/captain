use std::fmt;
use std::path::PathBuf;

use captain_core::ssh::{SshTarget, is_ssh};

use crate::ssh_tunnel::{close_ssh_tunnel, open_ssh_tunnel};

/// Where the Docker engine listens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Endpoint {
    /// A Unix socket path.
    Unix(PathBuf),
    /// A Windows named pipe, for example `//./pipe/docker_engine`.
    NamedPipe(String),
    /// A TCP address with scheme, for example `tcp://10.0.0.5:2375`.
    Tcp(String),
}

/// A host string that Captain cannot use.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("unsupported Docker host {0:?}")]
pub struct UnsupportedHost(pub String);

impl Endpoint {
    /// Parses a `DOCKER_HOST`-style string.
    pub fn parse(host: &str) -> Result<Self, UnsupportedHost> {
        if let Some(path) = host.strip_prefix("unix://") {
            Ok(Self::Unix(PathBuf::from(path)))
        } else if let Some(pipe) = host.strip_prefix("npipe://") {
            Ok(Self::NamedPipe(pipe.to_string()))
        } else if host.starts_with("tcp://") || host.starts_with("http://") {
            Ok(Self::Tcp(host.to_string()))
        } else {
            Err(UnsupportedHost(host.to_string()))
        }
    }

    /// The local endpoint for `host`. An `ssh://` host opens the SSH tunnel and gives
    /// its local socket; any other host stops the tunnel and parses as it is. This
    /// blocks while `ssh` logs in. The error is a message for the user.
    pub fn resolve(host: &str) -> Result<Self, String> {
        if is_ssh(host) {
            let target = SshTarget::parse(host).map_err(|err| err.to_string())?;
            return open_ssh_tunnel(&target).map(Self::Unix);
        }
        close_ssh_tunnel();
        Self::parse(host).map_err(|err| err.to_string())
    }
}

impl fmt::Display for Endpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unix(path) => write!(f, "unix://{}", path.display()),
            Self::NamedPipe(pipe) => write!(f, "npipe://{pipe}"),
            Self::Tcp(address) => f.write_str(address),
        }
    }
}

#[cfg(test)]
mod tests;
