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

/// The root shell script behind both commands. It checks again that the socket is
/// in the state the user saw, right before it changes it, so a change made while
/// the password prompt was open is left alone. Paths arrive as arguments, never as
/// script text: `$1` is `link` or `unlink`, `$2` the default socket, `$3` Captain
/// Engine's socket; for a link, `$4` is the state it replaces and `$5` that state's
/// link target.
const SCRIPT: &str = r#"s=$2
target() { l=$(readlink "$s") || return 1; case "$l" in /*) printf '%s' "$l" ;; *) printf '%s' "${s%/*}/$l" ;; esac; }
case "$1" in
unlink) [ -L "$s" ] && [ "$(target)" = "$3" ] ;;
link) case "$4" in
  missing) [ ! -e "$s" ] && [ ! -L "$s" ] ;;
  socket) [ -S "$s" ] && [ ! -L "$s" ] ;;
  link) [ -L "$s" ] && [ "$(target)" = "$5" ] ;;
  *) false ;;
  esac ;;
*) false ;;
esac || { echo "$s changed after Captain read it, so Captain left it. Try again." >&2; exit 3; }
if [ "$1" = unlink ]; then rm -f "$s"; else ln -sfn "$3" "$s"; fi"#;

/// The command that links `/var/run/docker.sock` to `target` as root, if the socket
/// is still what `replacing` says.
pub fn link_command(
    elevation: Elevation,
    target: &Path,
    replacing: &SocketLink,
) -> PrivilegedCommand {
    elevated(
        elevation,
        script_argv(Path::new(DEFAULT_SOCKET), target, Some(replacing)),
    )
}

/// The command that removes `/var/run/docker.sock` as root, if it still links to
/// Captain Engine's socket at `captain`.
pub fn unlink_command(elevation: Elevation, captain: &Path) -> PrivilegedCommand {
    elevated(
        elevation,
        script_argv(Path::new(DEFAULT_SOCKET), captain, None),
    )
}

/// The arguments of `/bin/sh -c` for [`SCRIPT`] on `socket`: a link that replaces
/// `replacing`, or with `None` an unlink.
fn script_argv(socket: &Path, captain: &Path, replacing: Option<&SocketLink>) -> Vec<String> {
    let mut argv = vec![SCRIPT.to_string(), "sh".into()];
    argv.push(
        if replacing.is_some() {
            "link"
        } else {
            "unlink"
        }
        .into(),
    );
    argv.push(socket.display().to_string());
    argv.push(captain.display().to_string());
    if let Some(replacing) = replacing {
        let (state, old) = match replacing {
            SocketLink::Missing => ("missing", String::new()),
            SocketLink::Socket => ("socket", String::new()),
            SocketLink::OtherLink(old) => ("link", old.display().to_string()),
            SocketLink::Captain | SocketLink::Other => ("none", String::new()),
        };
        argv.extend([state.to_string(), old]);
    }
    argv
}

/// Runs `/bin/sh -c` with `argv` as root.
fn elevated(elevation: Elevation, argv: Vec<String>) -> PrivilegedCommand {
    match elevation {
        // Each value is an argument of the run handler, and `quoted form of` quotes
        // it for the shell, so no path text becomes script text.
        Elevation::AppleScript => {
            let quoted = (1..=argv.len())
                .map(|ix| format!("quoted form of item {ix} of argv"))
                .collect::<Vec<_>>()
                .join(" & \" \" & ");
            let line =
                format!("do shell script \"/bin/sh -c \" & {quoted} with administrator privileges");
            let mut args = ["on run argv", &line, "end run"]
                .into_iter()
                .flat_map(|part| ["-e".to_string(), part.to_string()])
                .collect::<Vec<_>>();
            args.extend(argv);
            PrivilegedCommand {
                program: "/usr/bin/osascript",
                args,
            }
        }
        Elevation::Pkexec => PrivilegedCommand {
            program: "pkexec",
            args: ["/bin/sh".to_string(), "-c".into()]
                .into_iter()
                .chain(argv)
                .collect(),
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

#[cfg(test)]
mod tests;
