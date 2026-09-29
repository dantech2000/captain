//! An extension image before and after install.

use serde::{Deserialize, Serialize};

use super::{ExtensionLabels, ExtensionMetadata, host_platform};

/// A pulled image that is an extension, before the user confirms the install.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtensionCandidate {
    pub id: String,
    /// The image reference, for example `docker/disk-usage-extension:0.2.9`.
    pub image: String,
    /// The image ID, `sha256:…`. An update compares it.
    pub image_id: String,
    pub labels: ExtensionLabels,
    pub metadata: ExtensionMetadata,
}

impl ExtensionCandidate {
    /// The file names of the host binaries for this platform, for the install dialog.
    pub fn binary_names(&self) -> Vec<String> {
        self.metadata
            .host_binaries(host_platform())
            .iter()
            .map(|path| binary_name(path).to_string())
            .collect()
    }
}

/// What `extension.json` records about an installed extension.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstalledExtension {
    pub id: String,
    pub image: String,
    /// The image ID when Captain installed it. Empty for older installs.
    #[serde(default)]
    pub image_id: String,
    pub labels: ExtensionLabels,
    pub metadata: ExtensionMetadata,
    /// The file names in the extension's `bin` folder. `host.cli.exec` runs only these.
    #[serde(default)]
    pub binaries: Vec<String>,
    /// Unix seconds.
    #[serde(default)]
    pub installed: u64,
    /// The engine that runs the backend, as a `DOCKER_HOST` value or the `ssh://`
    /// URL. Empty for older installs, which every engine lists.
    #[serde(default)]
    pub engine: String,
}

impl InstalledExtension {
    pub fn new(candidate: ExtensionCandidate, installed: u64) -> Self {
        let binaries = candidate.binary_names();
        Self {
            id: candidate.id,
            image: candidate.image,
            image_id: candidate.image_id,
            labels: candidate.labels,
            metadata: candidate.metadata,
            binaries,
            installed,
            engine: String::new(),
        }
    }

    /// The image ID when Captain recorded one, else the reference. Install copies
    /// files from it and the backend runs it, so a pull that moves the tag later
    /// does not change the extension.
    pub fn pinned_image(&self) -> &str {
        match self.image_id.as_str() {
            "" => &self.image,
            id => id,
        }
    }

    /// Whether the backend runs on `engine`. An older install with no engine runs on
    /// every engine. See [`engine_key`].
    pub fn runs_on(&self, engine: &str) -> bool {
        self.engine.is_empty() || engine_key(&self.engine) == engine_key(engine)
    }

    /// The tab title, else the image title, else the image.
    pub fn title(&self) -> &str {
        match self.metadata.dashboard_tab() {
            Some(tab) if !tab.title.is_empty() => &tab.title,
            _ if !self.labels.title.is_empty() => &self.labels.title,
            _ => &self.image,
        }
    }

    /// The URL of the extension's page, served from its UI folder.
    pub fn page_url(&self) -> Option<String> {
        let tab = self.metadata.dashboard_tab()?;
        Some(format!(
            "{}://{}/{}",
            super::SCHEME,
            self.id,
            tab.src.trim_start_matches('/')
        ))
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }

    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|error| error.to_string())
    }
}

/// `host` with the path of a `unix://` socket resolved through symlinks, so two
/// paths to one socket name the same engine. Other hosts, and sockets that do not
/// exist, stay as they are.
pub fn engine_key(host: &str) -> String {
    host.strip_prefix("unix://")
        .and_then(|path| std::fs::canonicalize(path).ok())
        .map_or_else(
            || host.to_string(),
            |path| format!("unix://{}", path.display()),
        )
}

/// The last part of an image path: `/darwin/tool` is `tool`.
pub fn binary_name(path: &str) -> &str {
    path.rsplit(['/', '\\']).next().unwrap_or(path)
}

#[cfg(test)]
mod tests;
