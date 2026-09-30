//! Opens a project folder in the system's apps: the default app for folders, and a
//! terminal window.

use std::io;
use std::path::Path;
use std::process::Command;

/// Opens a terminal window in `dir`: Terminal on macOS, the default terminal on
/// Linux, and a Command Prompt on Windows. The shell gets the user's environment;
/// Captain cannot pass `DOCKER_HOST` through the system's launcher.
pub fn open_terminal(dir: &Path) -> io::Result<()> {
    terminal_command(dir).spawn().map(drop)
}

#[cfg(target_os = "macos")]
fn terminal_command(dir: &Path) -> Command {
    let mut command = Command::new("open");
    command.args(["-a", "Terminal"]).arg(dir);
    command
}

#[cfg(target_os = "windows")]
fn terminal_command(dir: &Path) -> Command {
    let mut command = Command::new("cmd");
    command.args(["/c", "start", "cmd"]).current_dir(dir);
    command
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn terminal_command(dir: &Path) -> Command {
    let mut command = Command::new("x-terminal-emulator");
    command.current_dir(dir);
    command
}
