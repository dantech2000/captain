use std::sync::Arc;

use super::{BUILD_CACHE_AGE, ReclaimItem, ReclaimTarget};
use crate::Engine;
use crate::format::bytes_label;
use crate::model::{ContainerAction, EngineInfo};

/// What a cleanup freed, and what the engine refused.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CleanupReport {
    pub freed_bytes: u64,
    /// One line per item that failed, with the engine's reason.
    pub failures: Vec<String>,
}

impl CleanupReport {
    /// For example `Freed 6.1 GB`, or `Freed 5 GB. 2 items were not removed: …`.
    pub fn summary(&self) -> String {
        let freed = format!("Freed {}", bytes_label(self.freed_bytes));
        match self.failures.as_slice() {
            [] => freed,
            [one] => format!("{freed}. 1 item was not removed: {one}"),
            [first, rest @ ..] => format!(
                "{freed}. {} items were not removed: {first}",
                rest.len() + 1
            ),
        }
    }
}

/// Removes `items`, which the preview read from the daemon `preview`. If `engine` is
/// not that daemon now, it removes nothing and returns why. Each item goes by its
/// ID or name: build cache by a prune filtered to the record, each stopped container
/// and each image by ID, each volume by name. It never forces, so the engine refuses
/// anything a container started to use since the preview.
pub async fn run_cleanup(
    engine: Arc<dyn Engine>,
    preview: &EngineInfo,
    items: Vec<ReclaimItem>,
) -> Result<CleanupReport, String> {
    match engine.info().await {
        Ok(info) if info.same_daemon(preview) => {}
        Ok(info) => {
            return Err(format!(
                "Captain is connected to a different engine ({}) than the preview \
                 listed, so nothing was removed.",
                info.endpoint
            ));
        }
        Err(error) => {
            return Err(format!(
                "Captain could not check which engine it is connected to, so nothing \
                 was removed: {error}"
            ));
        }
    }
    let mut report = CleanupReport::default();
    for item in &items {
        let result = match &item.target {
            ReclaimTarget::BuildCache { id } => {
                // A record a build used since the preview stays and frees nothing;
                // the report says so, so the freed total adds up.
                match engine.prune_build_record(id, BUILD_CACHE_AGE).await {
                    Ok(0) if item.size > 0 => {
                        Err("a build used it since the preview, so it stays".into())
                    }
                    Ok(bytes) => {
                        report.freed_bytes += bytes;
                        continue;
                    }
                    Err(error) => Err(error.to_string()),
                }
            }
            ReclaimTarget::Image { id } => remove_image(&engine, id).await,
            ReclaimTarget::Container { id } => engine
                .run_action(id, ContainerAction::Remove)
                .await
                .map_err(|error| error.to_string()),
            ReclaimTarget::Volume { name } => engine
                .remove_volume(name)
                .await
                .map_err(|error| error.to_string()),
        };
        match result {
            Ok(()) => report.freed_bytes += item.size,
            Err(error) => report.failures.push(format!("{}: {error}", item.name)),
        }
    }
    Ok(report)
}

/// Removes the image `id` if a fresh list still shows it with no container. By ID
/// and without force, the engine also refuses an image that a container uses, or
/// that tags in more than one repository point at; it never only untags it.
async fn remove_image(engine: &Arc<dyn Engine>, id: &str) -> Result<(), String> {
    let images = engine
        .list_images()
        .await
        .map_err(|error| format!("could not check it is unused: {error}"))?;
    match images.iter().find(|image| image.id == id) {
        None => return Err("it is no longer there".to_string()),
        Some(image) if image.in_use() => return Err("a container uses it now".to_string()),
        Some(_) => {}
    }
    engine
        .remove_image(id)
        .await
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests;
