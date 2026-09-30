//! Disk use and the prunes that the Storage page runs. See
//! docs/features/0031-storage.md.

use std::collections::HashMap;
use std::time::Duration;

use bollard::Docker;
use bollard::query_parameters::{
    DataUsageOptionsBuilder, ListContainersOptionsBuilder, ListImagesOptionsBuilder,
    ListVolumesOptions, PruneBuildOptionsBuilder,
};
use captain_core::EngineError;
use captain_core::model::DiskUsage;

use crate::mapping;

/// `docker system df -v`. An engine older than API 1.52 sends a format that bollard
/// does not read; then the lists give the items without build cache or totals.
pub(super) async fn disk_usage(docker: &Docker) -> Result<DiskUsage, EngineError> {
    let options = DataUsageOptionsBuilder::default().verbose(true).build();
    let response = docker
        .df(Some(options))
        .await
        .map_err(mapping::engine_error)?;
    match mapping::disk_usage(response) {
        Some(usage) => Ok(usage),
        None => from_lists(docker).await,
    }
}

async fn from_lists(docker: &Docker) -> Result<DiskUsage, EngineError> {
    let images = ListImagesOptionsBuilder::default().build();
    let containers = ListContainersOptionsBuilder::default()
        .all(true)
        .size(true)
        .build();
    let (images, containers, volumes) = futures::try_join!(
        docker.list_images(Some(images)),
        docker.list_containers(Some(containers)),
        docker.list_volumes(None::<ListVolumesOptions>),
    )
    .map_err(mapping::engine_error)?;
    let counts = mapping::image_usage(&containers);
    let mut usage = DiskUsage {
        images: images
            .into_iter()
            .map(|summary| mapping::image(summary, &counts))
            .collect(),
        containers: containers
            .into_iter()
            .map(mapping::disk_container)
            .collect(),
        volumes: volumes
            .volumes
            .unwrap_or_default()
            .into_iter()
            .map(|volume| mapping::volume(volume, &HashMap::new()))
            .collect(),
        ..DiskUsage::default()
    };
    // Shared layers count once per image here, so the images total is too high.
    usage.images_bytes = usage.images.iter().map(|image| image.size).sum();
    usage.containers_bytes = usage.containers.iter().map(|c| c.size_rw).sum();
    usage.volumes_bytes = usage.volumes.iter().filter_map(|v| v.size_bytes).sum();
    Ok(usage)
}

/// `docker builder prune --filter until=<hours>h`, without `all`, so internal and
/// frontend records stay. The `until` filter keeps records used in that time. See
/// https://docs.docker.com/reference/api/engine/version/v1.52/#tag/Image/operation/BuildPrune
pub(super) async fn prune_build_cache(
    docker: &Docker,
    older_than: Duration,
) -> Result<u64, EngineError> {
    let until = until(older_than);
    let filters = HashMap::from([("until", vec![until.as_str()])]);
    let options = PruneBuildOptionsBuilder::default()
        .filters(&filters)
        .build();
    let response = docker
        .prune_build(Some(options))
        .await
        .map_err(mapping::engine_error)?;
    Ok(reclaimed(response.space_reclaimed))
}

/// A Go duration in whole hours, for example `336h`.
fn until(older_than: Duration) -> String {
    format!("{}h", older_than.as_secs() / 3600)
}

fn reclaimed(bytes: Option<i64>) -> u64 {
    bytes.and_then(|n| u64::try_from(n).ok()).unwrap_or(0)
}
