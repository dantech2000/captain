//! Administrative access: what `/var/run/docker.sock` is now, what Captain may do with
//! it, and the privileged commands that link or unlink it. See feature 0015.

use std::path::{Path, PathBuf};

/// The socket that Docker tools use when `DOCKER_HOST` and the context are unset.
pub const DEFAULT_SOCKET: &str = "/var/run/docker.sock";

/// What the app found at [`DEFAULT_SOCKET`], without following links.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SocketProbe {
    Missing,
    /// A symbolic link, with its target as written.
    Link(PathBuf),
    /// A real Unix socket, so an engine listens there itself.
    Socket,
    /// A folder, a plain file, or something else.
    Other,
}

/// [`SocketProbe`] compared with Captain Engine's socket.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SocketLink {
    Missing,
    Captain,
    /// A link to another path, most likely another engine.
    OtherLink(PathBuf),
    Socket,
    Other,
}

/// What a click on the Administrative access button does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkAction {
    Link,
    Unlink,
}

impl SocketLink {
    /// Classifies `probe` for Captain Engine's socket at `captain`. A relative link
    /// target is read from `/var/run`, where the link lives.
    pub fn classify(probe: &SocketProbe, captain: &Path) -> Self {
        match probe {
            SocketProbe::Missing => Self::Missing,
            SocketProbe::Socket => Self::Socket,
            SocketProbe::Other => Self::Other,
            SocketProbe::Link(target) => {
                let target = Path::new(DEFAULT_SOCKET)
                    .parent()
                    .map_or_else(|| target.clone(), |dir| dir.join(target));
                if target == captain {
                    Self::Captain
                } else {
                    Self::OtherLink(target)
                }
            }
        }
    }

    /// The button's action. Captain never replaces a folder or a plain file.
    pub fn action(&self) -> Option<LinkAction> {
        match self {
            Self::Missing | Self::OtherLink(_) | Self::Socket => Some(LinkAction::Link),
            Self::Captain => Some(LinkAction::Unlink),
            Self::Other => None,
        }
    }

    /// True if linking replaces another engine's socket, so the user must confirm.
    pub fn needs_confirmation(&self) -> bool {
        matches!(self, Self::OtherLink(_) | Self::Socket)
    }

    /// One line for the Settings page.
    pub fn describe(&self) -> String {
        match self {
            Self::Missing => format!("{DEFAULT_SOCKET} does not exist."),
            Self::Captain => format!("{DEFAULT_SOCKET} points at Captain Engine."),
            Self::OtherLink(target) => format!(
                "{DEFAULT_SOCKET} points at {}, another engine.",
                target.display()
            ),
            Self::Socket => format!("{DEFAULT_SOCKET} is the socket of another engine."),
            Self::Other => format!("{DEFAULT_SOCKET} is not a socket, so Captain leaves it."),
        }
    }
}

/// The socket path of a `unix://` endpoint, or `None` for any other kind.
pub fn socket_path(endpoint: &str) -> Option<PathBuf> {
    endpoint
        .strip_prefix("unix://")
        .filter(|path| !path.is_empty())
        .map(PathBuf::from)
}

/// How the app asks for administrator rights.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Elevation {
    /// `osascript` with `do shell script … with administrator privileges` (macOS).
    AppleScript,
    /// `pkexec`, which asks through polkit (Linux).
    Pkexec,
}

/// A program and its arguments, run without a shell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrivilegedCommand {
    pub program: &'static str,
    pub args: Vec<String>,
}

/// The command that runs `ln -sfn target /var/run/docker.sock` as root.
pub fn link_command(elevation: Elevation, target: &Path) -> PrivilegedCommand {
    let target = target.display().to_string();
    match elevation {
        // The path is an argument of the run handler, and `quoted form of` quotes it
        // for the shell, so no path text becomes script text.
        Elevation::AppleScript => apple_script(
            &format!(
                "do shell script \"/bin/ln -sfn \" & quoted form of item 1 of argv & \" {DEFAULT_SOCKET}\" with administrator privileges"
            ),
            Some(target),
        ),
        Elevation::Pkexec => PrivilegedCommand {
            program: "pkexec",
            args: vec![
                "/bin/ln".into(),
                "-sfn".into(),
                target,
                DEFAULT_SOCKET.into(),
            ],
        },
    }
}

/// The command that removes `/var/run/docker.sock` as root.
pub fn unlink_command(elevation: Elevation) -> PrivilegedCommand {
    match elevation {
        Elevation::AppleScript => apple_script(
            &format!(
                "do shell script \"/bin/rm -f {DEFAULT_SOCKET}\" with administrator privileges"
            ),
            None,
        ),
        Elevation::Pkexec => PrivilegedCommand {
            program: "pkexec",
            args: vec!["/bin/rm".into(), "-f".into(), DEFAULT_SOCKET.into()],
        },
    }
}

/// A message for a failed privileged command, from its error output.
pub fn failure_message(stderr: &str) -> String {
    // AppleScript reports Cancel as error -128; polkit's pkexec exits with 126.
    if stderr.contains("-128") || stderr.contains("Request dismissed") {
        return "You canceled the request.".into();
    }
    match stderr.trim() {
        "" => "The command failed.".into(),
        message => message.to_string(),
    }
}

fn apple_script(line: &str, arg: Option<String>) -> PrivilegedCommand {
    let mut args = ["on run argv", line, "end run"]
        .into_iter()
        .flat_map(|part| ["-e".to_string(), part.to_string()])
        .collect::<Vec<_>>();
    args.extend(arg);
    PrivilegedCommand {
        program: "/usr/bin/osascript",
        args,
    }
}

#[cfg(test)]
mod tests;
