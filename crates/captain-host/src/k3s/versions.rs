//! The k3s versions to offer: the k3s channel server and GitHub's release list, as
//! Rancher Desktop reads them (k3sHelper.ts), with a cache for offline use.

use std::path::Path;

use captain_core::HostError;
use captain_core::kubernetes::{K3sVersion, VersionList, parse_channels, parse_releases};

use super::curl;
use crate::cancel::Cancel;

const CHANNELS_URL: &str = "https://update.k3s.io/v1-release/channels";
const RELEASES_URL: &str = "https://api.github.com/repos/k3s-io/k3s/releases?per_page=100";
/// Release candidates fill most of each page, so three pages reach back about a
/// year of stable releases.
const PAGES: u32 = 3;

/// The version list. `refresh` asks the network first; otherwise the cache file
/// answers when it exists. The versions in the download folder `cache` are always
/// in the list, so the picker works offline.
pub fn list(cache_file: &Path, cache: &Path, refresh: bool) -> VersionList {
    let cached = VersionList::load(cache_file);
    let mut list = match cached {
        Some(list) if !refresh => list,
        cached => match fetch() {
            Ok(list) => {
                if let Err(error) = list.save(cache_file) {
                    tracing::warn!(%error, "cannot cache the k3s versions");
                }
                list
            }
            Err(error) => {
                tracing::warn!(%error, "cannot fetch the k3s versions");
                cached.unwrap_or_default()
            }
        },
    };
    list.include(downloaded(cache));
    list
}

fn fetch() -> Result<VersionList, HostError> {
    let cancel = Cancel::default();
    let channels = parse_channels(&curl::text(CHANNELS_URL, &cancel)?).map_err(HostError)?;
    let mut releases = Vec::new();
    for page in 1..=PAGES {
        let json = curl::text(&format!("{RELEASES_URL}&page={page}"), &cancel)?;
        releases.extend(parse_releases(&json).map_err(HostError)?);
    }
    Ok(VersionList::new(releases, channels))
}

/// The versions with a complete folder in `cache`.
fn downloaded(cache: &Path) -> Vec<K3sVersion> {
    let Ok(entries) = std::fs::read_dir(cache) else {
        return Vec::new();
    };
    entries
        .flatten()
        .filter_map(|entry| entry.file_name().to_str()?.parse().ok())
        .collect()
}
