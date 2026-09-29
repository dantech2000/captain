//! Reads `limactl list --json`: one JSON object per line, one line per instance.
//! The fields follow `limatype.Instance` in lima-vm/lima.

use captain_core::{HostResources, HostStatus};
use serde::Deserialize;

/// The part of a Lima instance that Captain reads.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct LimaInstance {
    pub name: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub dir: String,
    #[serde(default)]
    pub cpus: u32,
    /// Bytes.
    #[serde(default)]
    pub memory: u64,
    /// Bytes.
    #[serde(default)]
    pub disk: u64,
    #[serde(default)]
    pub errors: Vec<String>,
}

impl LimaInstance {
    /// Lima's status as a [`HostStatus`].
    pub fn host_status(&self) -> HostStatus {
        match self.status.as_str() {
            "Running" => HostStatus::Running,
            "Stopped" => HostStatus::Stopped,
            "Installing" => HostStatus::Starting,
            "Broken" if !self.errors.is_empty() => HostStatus::Failed(self.errors.join("; ")),
            "Broken" => HostStatus::Failed("Captain Engine is broken.".into()),
            other => HostStatus::Failed(format!("Lima reports the status {other:?}.")),
        }
    }

    pub fn resources(&self) -> HostResources {
        HostResources {
            cpus: self.cpus,
            memory_bytes: self.memory,
            disk_bytes: self.disk,
        }
    }
}

/// Finds `name` in the output of `limactl list --json`. Lines that do not parse are
/// skipped, so one odd instance does not hide Captain's.
pub fn find_instance(json_lines: &str, name: &str) -> Option<LimaInstance> {
    json_lines
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| serde_json::from_str::<LimaInstance>(line).ok())
        .find(|instance| instance.name == name)
}

#[cfg(test)]
mod tests;
