//! Converts bollard volume types.

mod prune;
mod user;

use std::collections::{BTreeMap, HashMap};

use bollard::models::{SystemDataUsageResponse, Volume as DockerVolume, VolumeUsageData};
use captain_core::model::Volume;

pub use prune::{volume_prune, volume_prune_filters};
pub use user::volume_users;

const COMPOSE_PROJECT_LABEL: &str = "com.docker.compose.project";

/// Disk usage and container counts from `/system/df`, keyed by volume name.
pub type VolumeUsage = HashMap<String, VolumeUsageData>;

/// Reads the volume items of a `/system/df` response. The engine sends them as JSON
/// volume objects. Items that do not parse are left out, so their usage is unknown.
pub fn volume_usage(response: SystemDataUsageResponse) -> VolumeUsage {
    response
        .volume_usage
        .and_then(|usage| usage.items)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|item| serde_json::from_value::<DockerVolume>(item).ok())
        .filter_map(|volume| Some((volume.name, volume.usage_data?)))
        .collect()
}

pub fn volume(volume: DockerVolume, usage: &VolumeUsage) -> Volume {
    let data = usage
        .get(&volume.name)
        .cloned()
        .or(volume.usage_data.clone());
    // The engine reports -1 when it did not compute a value.
    let size_bytes = data.as_ref().and_then(|d| u64::try_from(d.size).ok());
    let containers = data
        .as_ref()
        .and_then(|d| usize::try_from(d.ref_count).ok());
    let labels: BTreeMap<_, _> = volume.labels.into_iter().collect();

    Volume {
        compose_project: labels.get(COMPOSE_PROJECT_LABEL).cloned(),
        name: volume.name,
        driver: volume.driver,
        mountpoint: volume.mountpoint,
        scope: volume.scope.map(|s| s.to_string()).unwrap_or_default(),
        created: volume.created_at.unwrap_or_default(),
        labels,
        options: volume.options.into_iter().collect(),
        size_bytes,
        containers,
    }
}

#[cfg(test)]
mod tests;
