//! `ssh://` engine URLs and the `ssh` command that forwards the remote Docker socket.
//! The URL rules follow the Docker CLI's `connhelper/ssh`. See
//! docs/features/0026-contexts-and-remote-hosts.md.

use std::fmt;
use std::path::Path;

/// The remote socket when the URL has no path.
pub const DEFAULT_REMOTE_SOCKET: &str = "/var/run/docker.sock";

/// A remote engine reached over SSH.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SshTarget {
    pub user: Option<String>,
    /// The host name or address, without IPv6 brackets.
    pub host: String,
    pub port: Option<u16>,
    /// The remote Docker socket, from the URL path.
    pub socket: Option<String>,
}

/// Why an `ssh://` URL cannot be used. The message is for the user.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("invalid SSH URL: {0}")]
pub struct SshUrlError(pub &'static str);

/// True if `host` is an `ssh://` URL.
pub fn is_ssh(host: &str) -> bool {
    host.starts_with("ssh://")
}

impl SshTarget {
    /// Parses `ssh://[user@]host[:port][/remote/socket]`.
    pub fn parse(url: &str) -> Result<Self, SshUrlError> {
        let rest = url
            .strip_prefix("ssh://")
            .ok_or(SshUrlError("it must start with ssh://"))?;
        if rest.contains('?') {
            return Err(SshUrlError("query parameters are not allowed"));
        }
        if rest.contains('#') {
            return Err(SshUrlError("fragments are not allowed"));
        }
        let (authority, path) = match rest.find('/') {
            Some(at) => rest.split_at(at),
            None => (rest, ""),
        };
        let (user, address) = match authority.rsplit_once('@') {
            Some((user, _)) if user.contains(':') => {
                return Err(SshUrlError("a password in the URL is not supported"));
            }
            Some(("", _)) => return Err(SshUrlError("the user name is empty")),
            Some((user, address)) => (Some(user.to_string()), address),
            None => (None, authority),
        };
        let (host, port) = split_port(address)?;
        if host.is_empty() {
            return Err(SshUrlError("the host name is empty"));
        }
        let unsafe_word = |word: &str| {
            word.starts_with('-') || word.chars().any(|c| c.is_whitespace() || c.is_control())
        };
        if unsafe_word(host) || user.as_deref().is_some_and(unsafe_word) {
            return Err(SshUrlError(
                "the user or host name has characters ssh cannot take",
            ));
        }
        let socket = (!path.trim_matches('/').is_empty()).then(|| path.to_string());
        Ok(Self {
            user,
            host: host.to_string(),
            port,
            socket,
        })
    }

    /// The Docker socket on the remote host.
    pub fn remote_socket(&self) -> &str {
        self.socket.as_deref().unwrap_or(DEFAULT_REMOTE_SOCKET)
    }

    /// The `ssh` arguments that forward `local` to the remote socket. `BatchMode`
    /// makes `ssh` fail instead of asking for a password.
    pub fn tunnel_args(&self, local: &Path) -> Vec<String> {
        let mut args: Vec<String> = [
            "-nNT",
            "-o",
            "BatchMode=yes",
            "-o",
            "ExitOnForwardFailure=yes",
            "-o",
            "StreamLocalBindUnlink=yes",
            "-o",
            "ConnectTimeout=30",
            "-o",
            "ServerAliveInterval=15",
            "-o",
            "ServerAliveCountMax=3",
            "-L",
        ]
        .map(String::from)
        .into();
        args.push(format!("{}:{}", local.display(), self.remote_socket()));
        if let Some(user) = &self.user {
            args.extend(["-l".into(), user.clone()]);
        }
        if let Some(port) = self.port {
            args.extend(["-p".into(), port.to_string()]);
        }
        args.extend(["--".into(), self.host.clone()]);
        args
    }

    /// A message for the user from the `stderr` of an `ssh` that exited early.
    pub fn explain_failure(&self, stderr: &str) -> String {
        let host = &self.host;
        if stderr.contains("Permission denied") {
            format!(
                "SSH cannot log in to {host} without a password. Add your key to the SSH \
                 agent (ssh-add) or set up key login, then try again."
            )
        } else if stderr.contains("Host key verification failed") {
            format!(
                "SSH does not know the host key of {host}. Run \"ssh {host}\" once in a \
                 terminal to check and save it, then try again."
            )
        } else {
            let line = stderr.lines().rev().find(|line| !line.trim().is_empty());
            match line {
                Some(line) => format!("SSH to {host} failed: {}", line.trim()),
                None => format!("SSH to {host} failed."),
            }
        }
    }
}

/// Splits `host[:port]` or `[v6]:port`.
fn split_port(address: &str) -> Result<(&str, Option<u16>), SshUrlError> {
    let (host, port) = match address.strip_prefix('[') {
        Some(bracketed) => {
            let (host, after) = bracketed
                .split_once(']')
                .ok_or(SshUrlError("the IPv6 address has no closing bracket"))?;
            match after {
                "" => (host, None),
                _ => (
                    host,
                    Some(
                        after
                            .strip_prefix(':')
                            .ok_or(SshUrlError("the port is not valid"))?,
                    ),
                ),
            }
        }
        None => match address.split_once(':') {
            Some((host, port)) => (host, Some(port)),
            None => (address, None),
        },
    };
    let port = match port {
        None | Some("") => None,
        Some(port) => Some(
            port.parse::<u16>()
                .ok()
                .filter(|port| *port != 0)
                .ok_or(SshUrlError("the port is not valid"))?,
        ),
    };
    Ok((host, port))
}

impl fmt::Display for SshTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ssh://")?;
        if let Some(user) = &self.user {
            write!(f, "{user}@")?;
        }
        if self.host.contains(':') {
            write!(f, "[{}]", self.host)?;
        } else {
            f.write_str(&self.host)?;
        }
        if let Some(port) = self.port {
            write!(f, ":{port}")?;
        }
        f.write_str(self.socket.as_deref().unwrap_or_default())
    }
}

#[cfg(test)]
mod tests;
