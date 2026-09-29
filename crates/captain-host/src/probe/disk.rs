use std::path::Path;
#[cfg(unix)]
use std::process::Command;
#[cfg(unix)]
use std::time::Duration;

/// Free bytes on the disk that holds `path`, from `df -Pk`. POSIX defines that
/// format on every Unix, so Captain needs no crate for it. `None` on Windows.
#[cfg(unix)]
pub fn free_space(path: &Path) -> Option<u64> {
    let mut command = Command::new("df");
    command.arg("-Pk").arg(path);
    let output = super::output_within(command, Duration::from_secs(5)).ok()?;
    parse_df(&String::from_utf8_lossy(&output.stdout))
}

#[cfg(not(unix))]
pub fn free_space(_: &Path) -> Option<u64> {
    None
}

/// Reads the "Available" column of `df -Pk` output, in 1024-byte blocks.
#[cfg(any(unix, test))]
pub fn parse_df(output: &str) -> Option<u64> {
    let line = output.lines().nth(1)?;
    let available: u64 = line.split_whitespace().nth(3)?.parse().ok()?;
    Some(available * 1024)
}

#[cfg(test)]
mod tests;
