//! Converts a `/system/df` response. API 1.52 and later group it by type, each with
//! its totals and, with `verbose`, its items. See
//! https://docs.docker.com/reference/api/engine/version/v1.52/#tag/System/operation/SystemDataUsage

use std::collections::HashMap;

use bollard::models::{
    BuildCache, ContainerSummary, ImageSummary, SystemDataUsageResponse, Volume as DockerVolume,
};
use captain_core::model::{BuildCacheRecord, DiskContainer, DiskUsage, parse_rfc3339};
use serde::de::DeserializeOwned;

use super::{container, image, image_usage, volume};

/// The disk use in `response`, or `None` when the engine sent the format of API
/// 1.51 and earlier, which bollard does not read.
pub fn disk_usage(response: SystemDataUsageResponse) -> Option<DiskUsage> {
    let images = response.image_usage?;
    let containers = response.container_usage.unwrap_or_default();
    let volumes = response.volume_usage.unwrap_or_default();
    let cache = response.build_cache_usage.unwrap_or_default();

    let container_items: Vec<ContainerSummary> = items(containers.items);
    let counts = image_usage(&container_items);
    Some(DiskUsage {
        images: items::<ImageSummary>(images.items)
            .into_iter()
            .map(|summary| image(summary, &counts))
            .collect(),
        containers: container_items.into_iter().map(disk_container).collect(),
        volumes: items::<DockerVolume>(volumes.items)
            .into_iter()
            .map(|v| volume(v, &HashMap::new()))
            .collect(),
        build_cache: items::<BuildCache>(cache.items)
            .into_iter()
            .map(build_cache_record)
            .collect(),
        images_bytes: bytes(images.total_size),
        containers_bytes: bytes(containers.total_size),
        volumes_bytes: bytes(volumes.total_size),
        build_cache_bytes: bytes(cache.total_size),
    })
}

/// One container with its writable layer and the volumes it mounts. The container
/// list gives `SizeRw` when it asks for sizes.
pub fn disk_container(mut summary: ContainerSummary) -> DiskContainer {
    let size_rw = summary.size_rw.and_then(|n| u64::try_from(n).ok());
    let image_id = summary.image_id.clone().unwrap_or_default();
    let volumes = summary
        .mounts
        .take()
        .unwrap_or_default()
        .into_iter()
        // Only volume mounts have a name.
        .filter_map(|mount| mount.name.filter(|name| !name.is_empty()))
        .collect();
    let container = container(summary);
    DiskContainer {
        id: container.id,
        name: container.name,
        image_id,
        state: container.state,
        status: container.status,
        created: container.created,
        size_rw: size_rw.unwrap_or(0),
        volumes,
    }
}

fn build_cache_record(record: BuildCache) -> BuildCacheRecord {
    BuildCacheRecord {
        id: record.id.unwrap_or_default(),
        description: record.description.unwrap_or_default(),
        kind: record.typ.map(|t| t.to_string()).unwrap_or_default(),
        size: record.size.and_then(|n| u64::try_from(n).ok()).unwrap_or(0),
        in_use: record.in_use.unwrap_or(false),
        shared: record.shared.unwrap_or(false),
        created: record
            .created_at
            .as_deref()
            .and_then(parse_rfc3339)
            .unwrap_or(0),
        last_used: record.last_used_at.as_deref().and_then(parse_rfc3339),
    }
}

/// The items that parse as `T`. The rest are left out.
fn items<T: DeserializeOwned>(values: Option<Vec<serde_json::Value>>) -> Vec<T> {
    values
        .unwrap_or_default()
        .into_iter()
        .filter_map(|value| serde_json::from_value(value).ok())
        .collect()
}

/// The engine reports -1 when it did not compute a size.
fn bytes(value: Option<i64>) -> u64 {
    value.and_then(|n| u64::try_from(n).ok()).unwrap_or(0)
}
