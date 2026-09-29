//! Checks the output of `limactl --version`.

use super::template::MINIMUM_LIMA_VERSION;

/// A version as `(major, minor, patch)`.
type Version = (u32, u32, u32);

/// Reads `limactl version 2.2.0` (also `v2.2.0` and `2.2.0-12-gabcdef`). `None` for
/// anything else, such as a development build.
pub fn parse_version(output: &str) -> Option<Version> {
    let word = output.split_whitespace().last()?;
    let word = word.strip_prefix('v').unwrap_or(word);
    let core = word.split(['-', '+']).next()?;
    let mut parts = core.split('.').map(|part| part.parse::<u32>().ok());
    let major = parts.next()??;
    let minor = parts.next()??;
    let patch = parts.next().flatten().unwrap_or(0);
    Some((major, minor, patch))
}

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
