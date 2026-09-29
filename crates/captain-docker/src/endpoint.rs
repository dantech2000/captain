use std::fmt;
use std::path::PathBuf;

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
