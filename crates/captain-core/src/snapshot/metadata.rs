use serde::{Deserialize, Serialize};

use crate::HostResources;
use crate::daemon::DaemonSettings;
use crate::kubernetes::KubernetesSettings;
use crate::settings::Settings;

/// The file in each snapshot folder that describes it.
pub const METADATA_FILE: &str = "metadata.json";
/// Written last. A folder without it is an incomplete snapshot.
pub const COMPLETE_FILE: &str = "complete.txt";

/// What `metadata.json` holds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotMetadata {
    pub name: String,
    #[serde(default)]
    pub description: String,
    /// Unix seconds.
    pub created: u64,
    #[serde(default)]
    pub captain_version: String,
    /// The instance's `lima-version` when the snapshot was made.
    #[serde(default)]
    pub lima_version: String,
    /// The bytes the disk used on the host. The disk is sparse, so this is less
    /// than its size.
    #[serde(default)]
    pub disk_allocated: u64,
    /// The engine's CPUs, memory, and disk size.
    pub resources: HostResources,
    /// The Docker daemon settings the engine had. `None` in snapshots from before
    /// Captain saved them.
    #[serde(default)]
    pub daemon: Option<DaemonSettings>,
    /// The Kubernetes settings the engine had, with its k3s version.
    #[serde(default)]
    pub kubernetes: Option<KubernetesSettings>,
}

impl SnapshotMetadata {
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }

    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|error| error.to_string())
    }

    /// Makes `settings` match the restored engine, so its next start does not edit
    /// `lima.yaml` back, change `daemon.json`, or move k3s to another version.
    pub fn adopt_into(&self, settings: &mut Settings) {
        settings.engine_resources = Some(self.resources);
        if let Some(daemon) = &self.daemon {
            settings.engine_daemon = daemon.clone();
        }
        if let Some(kubernetes) = &self.kubernetes {
            settings.kubernetes = kubernetes.clone();
        }
    }
}

/// One complete snapshot: its folder name and its metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    /// The folder name, a UUID.
    pub id: String,
    pub metadata: SnapshotMetadata,
}

/// Sorts the newest snapshot first.
pub fn newest_first(snapshots: &mut [Snapshot]) {
    snapshots.sort_by(|a, b| {
        b.metadata
            .created
            .cmp(&a.metadata.created)
            .then_with(|| a.metadata.name.cmp(&b.metadata.name))
    });
}

/// The snapshot whose name or ID is `key`. A name wins over an ID.
pub fn find<'a>(snapshots: &'a [Snapshot], key: &str) -> Option<&'a Snapshot> {
    snapshots
        .iter()
        .find(|snapshot| snapshot.metadata.name == key)
        .or_else(|| snapshots.iter().find(|snapshot| snapshot.id == key))
}

#[cfg(test)]
mod tests;
