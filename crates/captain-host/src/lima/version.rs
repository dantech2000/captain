//! Checks the output of `limactl --version`.

use captain_core::diagnostics::{MINIMUM_LIMA_VERSION, parse_version};

/// Fails with a message for the user when `output` names a Lima that is too old.
/// A version Captain cannot read passes, so development builds work.
pub fn check_version(output: &str) -> Result<(), String> {
    let (Some(found), Some(minimum)) = (parse_version(output), parse_version(MINIMUM_LIMA_VERSION))
    else {
        return Ok(());
    };
    if found >= minimum {
        return Ok(());
    }
    let (major, minor, patch) = found;
    Err(format!(
        "Lima {MINIMUM_LIMA_VERSION} or newer is required; this is {major}.{minor}.{patch}. \
         Update it with `brew upgrade lima`."
    ))
}

#[cfg(test)]
mod tests;
