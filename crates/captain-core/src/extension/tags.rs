//! The tag to pull when the user names none. Docker wants extension tags to follow
//! semver so that Docker Desktop can find the newest version
//! (<https://docs.docker.com/extensions/extensions-sdk/extensions/DISTRIBUTION/>).
//! Many extension repositories publish no `latest` tag at all. Captain picks the
//! newest version tag, as Rancher Desktop does (`findBestVersion` in
//! <https://github.com/rancher-sandbox/rancher-desktop/blob/main/pkg/rancher-desktop/main/extensions/manager.ts>),
//! and `latest` only when no tag is a version.

use semver::Version;

use crate::model::ImageReference;

/// The tag Captain pulls when no tag of the repository is a version.
pub const FALLBACK_TAG: &str = "latest";

/// The repository of `reference` when it names neither a tag nor a digest, for
/// example `docker/disk-usage-extension`.
pub fn untagged_repository(reference: &str) -> Option<String> {
    let reference = reference.trim();
    let last = reference.rsplit('/').next().unwrap_or(reference);
    if reference.contains('@') || last.contains(':') {
        return None;
    }
    ImageReference::parse(reference).map(|parsed| parsed.name)
}

/// The version a tag names: `1.2.3`, `v1.2.3`, or `v.1.2.3`, with an optional
/// pre-release such as `-rc.1`. `None` for other tags, such as `main` or `pr-214`.
pub fn tag_version(tag: &str) -> Option<Version> {
    let core = match tag.strip_prefix(['v', 'V']) {
        Some(rest) => rest.strip_prefix('.').unwrap_or(rest),
        None => tag,
    };
    Version::parse(core).ok()
}

/// The newest release tag, else the newest pre-release tag. `None` when no tag is a
/// version.
pub fn newest_version_tag<'a>(tags: impl IntoIterator<Item = &'a str>) -> Option<&'a str> {
    tags.into_iter()
        .filter_map(|tag| Some((tag_version(tag)?, tag)))
        .max_by(|(a, _), (b, _)| (a.pre.is_empty(), a).cmp(&(b.pre.is_empty(), b)))
        .map(|(_, tag)| tag)
}

/// The newest version tag, or [`FALLBACK_TAG`].
pub fn choose_tag(tags: &[String]) -> String {
    newest_version_tag(tags.iter().map(String::as_str))
        .unwrap_or(FALLBACK_TAG)
        .to_string()
}

#[cfg(test)]
mod tests;
