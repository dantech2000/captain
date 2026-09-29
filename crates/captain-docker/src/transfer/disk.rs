//! Free space on the target engine's disk, read with `df` in a helper container. The
//! helper's root file system lives on the engine's data disk, where copies land.

use bollard::Docker;
use captain_core::EngineError;

use super::helper;

/// Prints the POSIX `df` line for the helper's root, in KiB blocks.
const DF_SCRIPT: &str = "df -Pk / | tail -n 1";

/// Free bytes on the target's data disk.
pub async fn free_space(target: &Docker) -> Result<u64, EngineError> {
    helper::ensure_image(target).await?;
    let output = helper::run_script(target, "df", None, DF_SCRIPT).await?;
    parse_df(&output)
        .ok_or_else(|| EngineError::Api(format!("cannot read df output: {}", output.trim())))
}

/// Reads the available column of a `df -Pk` line, in bytes.
fn parse_df(line: &str) -> Option<u64> {
    let available: u64 = line.split_whitespace().nth(3)?.parse().ok()?;
    available.checked_mul(1024)
}

#[cfg(test)]
mod tests;
