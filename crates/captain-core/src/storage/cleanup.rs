use std::sync::Arc;

use super::{BUILD_CACHE_AGE, ReclaimItem, ReclaimTarget};
use crate::Engine;
use crate::format::bytes_label;
use crate::model::ContainerAction;

/// What a cleanup freed, and what the engine refused.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CleanupReport {
    pub freed_bytes: u64,
    /// One line per item or prune that failed, with the engine's reason.
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

/// Removes `items`: one build prune with the plan's age filter, then each
/// stopped container by ID, each image tag by tag, and each volume by name. It never forces,
/// so the engine refuses anything a container started to use since the preview.
pub async fn run_cleanup(engine: Arc<dyn Engine>, items: Vec<ReclaimItem>) -> CleanupReport {
    let mut report = CleanupReport::default();
    let has = |target: ReclaimTarget| items.iter().any(|item| item.target == target);
    if has(ReclaimTarget::BuildCache) {
        match engine.prune_build_cache(BUILD_CACHE_AGE).await {
            Ok(bytes) => report.freed_bytes += bytes,
            Err(error) => report.failures.push(format!("build cache: {error}")),
        }
    }
    for item in &items {
        let result = match &item.target {
            ReclaimTarget::Image { id, tags } => remove_image(&engine, id, tags).await,
            ReclaimTarget::Container { id } => engine.run_action(id, ContainerAction::Remove).await,
            ReclaimTarget::Volume { name } => engine.remove_volume(name).await,
            ReclaimTarget::BuildCache => continue,
        };
        match result {
            Ok(()) => report.freed_bytes += item.size,
            Err(error) => report.failures.push(format!("{}: {error}", item.name)),
        }
    }
    report
}

/// Removing an image by ID fails while it has more than one tag, so each tag goes
/// in turn. Removing the last tag removes the image.
async fn remove_image(
    engine: &Arc<dyn Engine>,
    id: &str,
    tags: &[String],
) -> Result<(), crate::EngineError> {
    if tags.is_empty() {
        return engine.remove_image(id).await;
    }
    for tag in tags {
        engine.remove_image(tag).await?;
    }
    Ok(())
}
