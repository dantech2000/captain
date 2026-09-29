use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// The oldest k3s that Captain offers. Older releases bundle a cri-dockerd that
/// cannot read images from Docker 25 and newer (k3s-io/k3s#9279); the fix shipped
/// in the February 2024 releases.
pub const MINIMUM_VERSION: &str = "v1.29.2+k3s1";

/// A k3s release tag such as `v1.36.4+k3s1` or `v1.37.1-rc2+k3s1`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct K3sVersion {
    raw: String,
    numbers: [u32; 3],
    /// `rc2` in `v1.37.1-rc2+k3s1`. Only a version without it is stable.
    pre: Option<String>,
    /// `1` in `+k3s1`.
    build: u32,
}

impl K3sVersion {
    pub fn minimum() -> Self {
        MINIMUM_VERSION.parse().expect("the minimum version parses")
    }

    pub fn is_stable(&self) -> bool {
        self.pre.is_none()
    }

    /// `v1.36`, the name of this version's minor channel.
    pub fn minor_channel(&self) -> String {
        format!("v{}.{}", self.numbers[0], self.numbers[1])
    }

    pub fn as_str(&self) -> &str {
        &self.raw
    }
}

impl FromStr for K3sVersion {
    type Err = String;

    fn from_str(text: &str) -> Result<Self, String> {
        let bad = || format!("{text:?} is not a k3s version such as v1.36.4+k3s1.");
        let rest = text.trim().strip_prefix('v').ok_or_else(bad)?;
        let (core, build) = rest.split_once("+k3s").ok_or_else(bad)?;
        let digits = |part: &str| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit());
        let (core, pre) = match core.split_once('-') {
            // The tag names a cache folder, so it stays one safe path component.
            Some((_, pre))
                if pre.is_empty()
                    || !pre.chars().all(|c| c.is_ascii_alphanumeric() || c == '.') =>
            {
                return Err(bad());
            }
            Some((core, pre)) => (core, Some(pre.to_string())),
            None => (core, None),
        };
        if !digits(build) || !core.split('.').all(digits) {
            return Err(bad());
        }
        let build = build.parse().map_err(|_| bad())?;
        let parts: Vec<u32> = core
            .split('.')
            .map(str::parse)
            .collect::<Result<_, _>>()
            .map_err(|_| bad())?;
        let numbers: [u32; 3] = parts.try_into().map_err(|_| bad())?;
        Ok(Self {
            raw: text.trim().to_string(),
            numbers,
            pre,
            build,
        })
    }
}

impl TryFrom<String> for K3sVersion {
    type Error = String;

    fn try_from(text: String) -> Result<Self, String> {
        text.parse()
    }
}

impl From<K3sVersion> for String {
    fn from(version: K3sVersion) -> Self {
        version.raw
    }
}

impl fmt::Display for K3sVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.raw)
    }
}

impl Ord for K3sVersion {
    /// By number, then a release candidate before the release, then by k3s build.
    fn cmp(&self, other: &Self) -> Ordering {
        self.numbers
            .cmp(&other.numbers)
            .then_with(|| match (&self.pre, &other.pre) {
                (None, None) => Ordering::Equal,
                (None, Some(_)) => Ordering::Greater,
                (Some(_), None) => Ordering::Less,
                (Some(a), Some(b)) => a.cmp(b),
            })
            .then(self.build.cmp(&other.build))
    }
}

impl PartialOrd for K3sVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests;
