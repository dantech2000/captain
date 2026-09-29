use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::K3sVersion;

/// The k3s versions the picker offers, newest first, and the channel labels. It is
/// also the cache file, `~/.captain/cache/k3s-versions.json`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct VersionList {
    pub versions: Vec<K3sVersion>,
    /// The newest version of each channel, such as `stable`, `latest`, or `v1.36`.
    pub channels: BTreeMap<String, K3sVersion>,
}

impl VersionList {
    /// The stable releases at or above [`super::MINIMUM_VERSION`], newest first.
    pub fn new(releases: Vec<K3sVersion>, channels: BTreeMap<String, K3sVersion>) -> Self {
        let minimum = K3sVersion::minimum();
        let mut versions: Vec<K3sVersion> = releases
            .into_iter()
            .filter(|version| version.is_stable() && *version >= minimum)
            .collect();
        versions.sort_by(|a, b| b.cmp(a));
        versions.dedup();
        Self { versions, channels }
    }

    /// The version of the `stable` channel, the default when Kubernetes turns on.
    pub fn stable(&self) -> Option<&K3sVersion> {
        self.channels.get("stable")
    }

    /// `v1.36.4+k3s1 (stable, v1.36)`: the version with the channels it leads.
    pub fn label(&self, version: &K3sVersion) -> String {
        let mut names: Vec<&str> = self
            .channels
            .iter()
            .filter(|(_, latest)| *latest == version)
            .map(|(name, _)| name.as_str())
            .collect();
        // The named channels first, then the minor channel.
        names.sort_by_key(|name| (name.starts_with('v'), *name));
        if names.is_empty() {
            version.to_string()
        } else {
            format!("{version} ({})", names.join(", "))
        }
    }

    /// Adds versions that are not in the list, such as the downloaded ones when the
    /// list comes from an old cache.
    pub fn include(&mut self, extra: impl IntoIterator<Item = K3sVersion>) {
        let channels = std::mem::take(&mut self.channels);
        let all = std::mem::take(&mut self.versions).into_iter().chain(extra);
        *self = Self::new(all.collect(), channels);
    }

    /// The cached list, or `None` when there is none or it cannot be read.
    pub fn load(path: &Path) -> Option<Self> {
        serde_json::from_str(&std::fs::read_to_string(path).ok()?).ok()
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let json = serde_json::to_string_pretty(self).unwrap_or_default();
        std::fs::write(path, json)
    }
}

/// Reads `https://update.k3s.io/v1-release/channels`. It keeps `stable`, `latest`,
/// and the minor channels such as `v1.36`, and skips the testing channels.
pub fn parse_channels(json: &str) -> Result<BTreeMap<String, K3sVersion>, String> {
    let value: Value = serde_json::from_str(json).map_err(|error| error.to_string())?;
    let data = value["data"]
        .as_array()
        .ok_or("The channel list has no data.")?;
    Ok(data
        .iter()
        .filter_map(|channel| {
            let name = channel["name"].as_str()?;
            let latest: K3sVersion = channel["latest"].as_str()?.parse().ok()?;
            let wanted = matches!(name, "stable" | "latest")
                || (name.starts_with("v1.") && !name.contains('-'));
            (wanted && latest.is_stable()).then(|| (name.to_string(), latest))
        })
        .collect())
}

/// Reads one page of `https://api.github.com/repos/k3s-io/k3s/releases`. Drafts,
/// prereleases, and tags that are not k3s versions are skipped.
pub fn parse_releases(json: &str) -> Result<Vec<K3sVersion>, String> {
    let value: Value = serde_json::from_str(json).map_err(|error| error.to_string())?;
    let releases = value.as_array().ok_or("The release list is not a list.")?;
    Ok(releases
        .iter()
        .filter(|release| release["draft"] != true && release["prerelease"] != true)
        .filter_map(|release| release["tag_name"].as_str()?.parse().ok())
        .collect())
}

#[cfg(test)]
mod tests;
