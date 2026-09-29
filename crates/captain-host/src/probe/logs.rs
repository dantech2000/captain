use std::path::Path;

/// The total size of the `*.log` files in `dir`: Lima's host agent logs
/// (`ha.stdout.log`, `ha.stderr.log`) and serial logs. `None` if `dir` does not
/// exist.
pub fn log_bytes(dir: &Path) -> Option<u64> {
    let entries = std::fs::read_dir(dir).ok()?;
    let total = entries
        .flatten()
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "log"))
        .filter_map(|entry| entry.metadata().ok())
        .filter(|meta| meta.is_file())
        .map(|meta| meta.len())
        .sum();
    Some(total)
}

#[cfg(test)]
mod tests;
