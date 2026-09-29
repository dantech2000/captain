//! Reads what the source engine holds and builds the plan.

mod items;

use std::path::Path;

use captain_core::EngineError;
use captain_core::migration::MigrationPlan;

use super::source::SourceEngine;
use crate::mapping;

/// Lists networks, volumes, images, and containers in the source. `label` names the
/// source in the plan.
pub async fn scan(source: &SourceEngine, label: &str) -> Result<MigrationPlan, EngineError> {
    let (networks, volumes, images, containers) = futures::join!(
        source.list_networks(),
        source.list_volumes(),
        source.list_images(),
        source.list_containers(),
    );
    let containers = containers?;
    // Sizes are extra. Without them the plan still shows, with unknown sizes.
    let usage = source.disk_usage().await.map(mapping::volume_usage);
    let usage = usage.unwrap_or_else(|error| {
        tracing::warn!(%error, "could not read volume sizes");
        mapping::VolumeUsage::default()
    });
    let image_usage = mapping::image_usage(&containers);
    let volumes = volumes?
        .into_iter()
        .map(|volume| mapping::volume(volume, &usage))
        .collect();
    let images = images?
        .into_iter()
        .map(|summary| mapping::image(summary, &image_usage))
        .collect();

    let mut all = items::networks(networks?);
    all.extend(items::volumes(volumes));
    all.extend(items::images(images));
    all.extend(items::containers(containers, Path::exists));
    Ok(MigrationPlan::new(label, all))
}
