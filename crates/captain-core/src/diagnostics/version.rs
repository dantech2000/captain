//! Reads tool versions such as the output of `limactl --version`.

/// Lima 2.2.0 fixed a guest agent leak that silently killed port forwarding after
/// many connections (lima-vm/lima#5210). Captain Engine refuses older versions.
pub const MINIMUM_LIMA_VERSION: &str = "2.2.0";

/// A version as `(major, minor, patch)`.
pub type Version = (u32, u32, u32);

/// Reads the last word of `output` as a version: `limactl version 2.2.0`, also
/// `v2.2.0` and `2.2.0-12-gabcdef`. `None` for anything else, such as a
/// development build.
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

#[cfg(test)]
mod tests;
