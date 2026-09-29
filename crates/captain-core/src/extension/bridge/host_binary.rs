//! `host.cli.exec` runs only the extension's own binaries.

use std::path::{Path, PathBuf};

/// The binary `cmd` in `bin_dir`, if it is one of `installed`, the names Captain
/// copied at install. Paths, and names the extension did not ship, are refused.
pub fn host_binary(bin_dir: &Path, installed: &[String], cmd: &str) -> Result<PathBuf, String> {
    let plain = !cmd.is_empty() && !cmd.contains(['/', '\\']) && cmd != "." && cmd != "..";
    let name = [cmd.to_string(), format!("{cmd}.exe")]
        .into_iter()
        .find(|name| plain && installed.contains(name));
    name.map(|name| bin_dir.join(name))
        .ok_or_else(|| format!("{cmd} is not a host binary of this extension"))
}

#[cfg(test)]
mod tests;
