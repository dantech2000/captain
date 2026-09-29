use serde::{Deserialize, Serialize};

use crate::HostResources;

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
}

impl SnapshotMetadata {
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }

    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|error| error.to_string())
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
