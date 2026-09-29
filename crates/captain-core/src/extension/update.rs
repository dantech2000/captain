//! Updating an installed extension: the image to pull, and how its version compares
//! with the installed one.

use std::cmp::Ordering;

use super::{ExtensionCandidate, InstalledExtension};
use crate::model::ImageReference;

/// The tag an update pulls when the user names none.
pub const DEFAULT_UPDATE_TAG: &str = "latest";

/// The installed image's repository with `tag`, or with [`DEFAULT_UPDATE_TAG`] when
/// `tag` is empty. `None` for a tag that is not valid in a reference.
pub fn update_reference(image: &str, tag: &str) -> Option<String> {
    let tag = match tag.trim() {
        "" => DEFAULT_UPDATE_TAG,
        tag => tag,
    };
    if tag.contains(['/', ':', '@']) {
        return None;
    }
    let repository = ImageReference::parse(image)?.name;
    let repository = repository.split('@').next().unwrap_or_default();
    ImageReference::parse(&format!("{repository}:{tag}")).map(|r| r.to_string())
}

/// What a check for an update found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateCheck {
    /// The pulled image is the installed one.
    UpToDate { image: String },
    /// The pulled image differs. The user confirms before Captain reinstalls.
    Available(Box<ExtensionUpdate>),
}

/// An installed extension and the image that would replace it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtensionUpdate {
    pub extension: InstalledExtension,
    pub candidate: ExtensionCandidate,
}

impl ExtensionUpdate {
    /// `Some(Ordering::Less)` when the new image says it is an older version than the
    /// installed one. `None` when either has no version label, or a label that is not
    /// a dotted number.
    pub fn version_order(&self) -> Option<Ordering> {
        compare_versions(
            &self.candidate.labels.version,
            &self.extension.labels.version,
        )
    }
}

/// Compares two versions such as `0.2.9` and `v0.10.0` part by part as numbers. A
/// suffix after `-` or `+` is ignored. `None` if either is not a dotted number.
pub fn compare_versions(a: &str, b: &str) -> Option<Ordering> {
    let parts = |version: &str| -> Option<Vec<u64>> {
        let core = version.trim().trim_start_matches('v');
        let core = core.split(['-', '+']).next()?;
        core.split('.').map(|part| part.parse().ok()).collect()
    };
    let (a, b) = (parts(a)?, parts(b)?);
    let len = a.len().max(b.len());
    let at = |v: &[u64], i: usize| v.get(i).copied().unwrap_or(0);
    Some(
        (0..len)
            .map(|i| at(&a, i).cmp(&at(&b, i)))
            .find(|order| order.is_ne())
            .unwrap_or(Ordering::Equal),
    )
}

#[cfg(test)]
mod tests;
