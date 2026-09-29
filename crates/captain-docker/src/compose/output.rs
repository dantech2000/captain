//! Reads what the `docker compose` CLI prints.

/// The version from `docker compose version --format json`, which prints
/// `{"version":"v2.29.1"}`. `None` if the output is not that shape.
pub fn parse_version(stdout: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(stdout.trim()).ok()?;
    let version = value.get("version")?.as_str()?.trim();
    (!version.is_empty()).then(|| version.to_string())
}

/// The message to show for a failed command. Compose prints progress lines and the
/// error on stderr, so this prefers the last line that mentions an error, then the
/// last non-empty line.
pub fn error_message(stderr: &str) -> Option<String> {
    let lines: Vec<&str> = stderr
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();
    lines
        .iter()
        .rev()
        .find(|line| line.to_lowercase().contains("error"))
        .or_else(|| lines.last())
        .map(|line| line.to_string())
}

#[cfg(test)]
mod tests;
