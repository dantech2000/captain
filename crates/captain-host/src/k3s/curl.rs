//! Downloads with the system `curl`, which macOS always has.

use std::path::Path;
use std::process::Command;

use captain_core::HostError;

/// GitHub's API refuses requests without a user agent.
const USER_AGENT: &str = concat!("Captain/", env!("CARGO_PKG_VERSION"));

/// The body of `url`.
pub fn text(url: &str) -> Result<String, HostError> {
    let output = command(url)
        .output()
        .map_err(|error| HostError(format!("Cannot run curl: {error}")))?;
    if !output.status.success() {
        return Err(failed(url, &output.stderr));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Saves `url` to `file`.
pub fn save(url: &str, file: &Path) -> Result<(), HostError> {
    let output = command(url)
        .arg("--output")
        .arg(file)
        .output()
        .map_err(|error| HostError(format!("Cannot run curl: {error}")))?;
    if !output.status.success() {
        return Err(failed(url, &output.stderr));
    }
    Ok(())
}

fn command(url: &str) -> Command {
    let mut command = Command::new("curl");
    command
        .args(["--fail", "--silent", "--show-error", "--location"])
        .args(["--retry", "2", "--connect-timeout", "20"])
        .args(["--user-agent", USER_AGENT])
        .arg(url);
    command
}

fn failed(url: &str, stderr: &[u8]) -> HostError {
    let why = String::from_utf8_lossy(stderr);
    HostError(format!("Cannot download {url}. {}", why.trim()))
}
