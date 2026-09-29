//! Volume models: the list entry, the containers that use a volume, and prune results.

mod prune;
mod user;

use std::collections::BTreeMap;

pub use prune::VolumePrune;
pub use user::VolumeUser;

/// The label the engine puts on volumes created without a name.
const ANONYMOUS_LABEL: &str = "com.docker.volume.anonymous";

/// A named volume as Captain shows it in lists.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Volume {
    pub name: String,
    /// The volume driver, for example `local`.
    pub driver: String,
    /// Where the volume lives on the engine host.
    pub mountpoint: String,
    /// `local` or `global`. Empty if unknown.
    pub scope: String,
    /// Creation time in RFC 3339 format, as the engine reports it. Empty if unknown.
    pub created: String,
    pub labels: BTreeMap<String, String>,
    /// Driver options, for example `type` and `device` for a bind-backed volume.
    pub options: BTreeMap<String, String>,
    /// Disk usage in bytes. `None` if the engine did not report it.
    pub size_bytes: Option<u64>,
    /// The number of containers, running or stopped, that use the volume.
    /// `None` if the engine did not report it.
    pub containers: Option<usize>,
    /// The Compose project, from the `com.docker.compose.project` label.
    pub compose_project: Option<String>,
}

impl Volume {
    /// Anonymous volumes have a 64-character hex name.
    pub fn is_anonymous(&self) -> bool {
        self.name.len() == 64 && self.name.bytes().all(|b| b.is_ascii_hexdigit())
    }

    /// True if a prune without `all` removes the volume when no container uses it.
    /// Docker API 1.42 and later prune only anonymous volumes by default. The engine
    /// marks them with the `com.docker.volume.anonymous` label.
    pub fn pruned_by_default(&self) -> bool {
        self.labels.contains_key(ANONYMOUS_LABEL) || self.is_anonymous()
    }

    /// The name for lists: anonymous volumes are cut to 12 characters, like an ID.
    pub fn display_name(&self) -> &str {
        if self.is_anonymous() {
            &self.name[..12]
        } else {
            &self.name
        }
    }

    /// True if at least one container uses the volume.
    pub fn is_in_use(&self) -> bool {
        self.containers.is_some_and(|n| n > 0)
    }

    /// True if the engine reports that no container uses the volume.
    pub fn is_unused(&self) -> bool {
        self.containers == Some(0)
    }

    /// Captain offers Remove only for volumes that no container uses. If the count is
    /// unknown, the engine decides.
    pub fn can_remove(&self) -> bool {
        !self.is_in_use()
    }

    /// The date part of [`Volume::created`], for example `2026-09-28`.
    pub fn created_date(&self) -> &str {
        self.created.get(..10).unwrap_or(&self.created)
    }

    /// For example `2 containers`, `Unused`, or `Unknown`.
    pub fn usage_label(&self) -> String {
        match self.containers {
            None => "Unknown".into(),
            Some(0) => "Unused".into(),
            Some(1) => "1 container".into(),
            Some(n) => format!("{n} containers"),
        }
    }
}

#[cfg(test)]
mod tests;
