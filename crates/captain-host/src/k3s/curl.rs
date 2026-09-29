//! Downloads with the system `curl`, which macOS always has. Each runs under a
//! [`Cancel`], so a stop ends it, and a stalled transfer fails on its own.

use std::path::Path;
use std::process::Command;

use captain_core::HostError;

use crate::cancel::Cancel;

/// GitHub's API refuses requests without a user agent.
const USER_AGENT: &str = concat!("Captain/", env!("CARGO_PKG_VERSION"));
/// A transfer slower than this many bytes per second for [`STALL_SECONDS`] fails.
/// `--connect-timeout` covers only the connection.
const STALL_BYTES: &str = "1000";
const STALL_SECONDS: &str = "60";
/// The longest a small text download may take.
const TEXT_SECONDS: &str = "120";

/// The body of `url`.
pub fn text(url: &str, cancel: &Cancel) -> Result<String, HostError> {
    let mut command = command(url);
    command.args(["--max-time", TEXT_SECONDS]);
    let output = cancel.output(command, None)?;
    if !output.status.success() {
        return Err(failed(url, &output.stderr));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Saves `url` to `file`.
pub fn save(url: &str, file: &Path, cancel: &Cancel) -> Result<(), HostError> {
    let mut command = command(url);
    command.arg("--output").arg(file);
    let output = cancel.output(command, None)?;
    if !output.status.success() {
        return Err(failed(url, &output.stderr));
    }
    Ok(())
}

/// True when k3s answers `/ping` on `127.0.0.1:port`. The route needs no login
/// (k3s pkg/server/handlers/router.go). curl trusts only `ca`, k3s's own authority,
/// so another cluster or program on the port does not count.
pub fn ping(port: u16, ca: &Path, cancel: &Cancel) -> bool {
    let mut command = Command::new("curl");
    command
        .args(["--silent", "--max-time", "5", "--cacert"])
        .arg(ca)
        .arg(format!("https://127.0.0.1:{port}/ping"));
    cancel
        .output(command, None)
        .is_ok_and(|output| output.status.success() && output.stdout == b"pong")
}

fn command(url: &str) -> Command {
    let mut command = Command::new("curl");
    command
        .args(["--fail", "--silent", "--show-error", "--location"])
        .args(["--retry", "2", "--connect-timeout", "20"])
        .args(["--speed-limit", STALL_BYTES, "--speed-time", STALL_SECONDS])
        .args(["--user-agent", USER_AGENT])
        .arg(url);
    command
}

fn failed(url: &str, stderr: &[u8]) -> HostError {
    let why = String::from_utf8_lossy(stderr);
    HostError(format!("Cannot download {url}. {}", why.trim()))
}
