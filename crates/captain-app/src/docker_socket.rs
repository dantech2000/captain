//! Administrative access on macOS and Linux: reads `/var/run/docker.sock`, and links
//! or unlinks it through the system password prompt. The decisions and the command
//! lines are in `captain_core::behavior::docker_socket`. See feature 0015.

use std::fs;
use std::os::unix::fs::FileTypeExt;
use std::path::Path;
use std::process::Command;

use captain_core::behavior::docker_socket::{
    DEFAULT_SOCKET, Elevation, PrivilegedCommand, SocketProbe, failure_message, link_command,
    unlink_command,
};

const ELEVATION: Elevation = if cfg!(target_os = "macos") {
    Elevation::AppleScript
} else {
    Elevation::Pkexec
};

/// What `/var/run/docker.sock` is, without following a link.
pub fn probe() -> SocketProbe {
    let path = Path::new(DEFAULT_SOCKET);
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return SocketProbe::Missing;
    };
    let kind = metadata.file_type();
    if kind.is_symlink() {
        fs::read_link(path).map_or(SocketProbe::Other, SocketProbe::Link)
    } else if kind.is_socket() {
        SocketProbe::Socket
    } else {
        SocketProbe::Other
    }
}

pub fn link(target: &Path) -> Result<(), String> {
    run(link_command(ELEVATION, target))
}

pub fn unlink() -> Result<(), String> {
    run(unlink_command(ELEVATION))
}

/// Runs `command` and waits for it, which includes the password prompt.
fn run(command: PrivilegedCommand) -> Result<(), String> {
    tracing::info!(program = command.program, "asking for administrator rights");
    let output = Command::new(command.program)
        .args(&command.args)
        .output()
        .map_err(|error| format!("Captain cannot run {}. {error}", command.program))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(failure_message(&String::from_utf8_lossy(&output.stderr)))
    }
}
