//! Converts bollard image types.

mod detail;
mod push;
mod run;

use std::collections::HashMap;

use bollard::errors::Error;
use bollard::models::{ContainerSummary, CreateImageInfo, ImageSummary};
use captain_core::EngineError;
use captain_core::model::{Image, PullProgress};

pub use detail::{image_detail, image_history};
pub use push::{credentials, push_progress};
pub use run::{restart_name, run_body};

use super::engine_error;

/// The tag the engine lists for an untagged image.
const UNTAGGED: &str = "<none>:<none>";

/// Converts one image. `usage` counts containers by image ID. It is used when the
/// engine does not report the count itself, which it marks with -1.
pub fn image(summary: ImageSummary, usage: &HashMap<String, usize>) -> Image {
    let repo_tags: Vec<String> = summary
        .repo_tags
        .into_iter()
        .filter(|tag| tag != UNTAGGED)
        .collect();
    let containers = match usize::try_from(summary.containers) {
        Ok(count) => count,
        Err(_) => usage.get(&summary.id).copied().unwrap_or(0),
    };
    Image {
        dangling: repo_tags.is_empty(),
        id: summary.id,
        repo_tags,
        size: u64::try_from(summary.size).unwrap_or(0),
        created: summary.created,
        containers,
    }
}

/// True if any summary lacks a container count, so the caller must count containers.
pub fn needs_usage(summaries: &[ImageSummary]) -> bool {
    summaries.iter().any(|summary| summary.containers < 0)
}

/// Counts containers, running or stopped, by the ID of their image.
pub fn image_usage(containers: &[ContainerSummary]) -> HashMap<String, usize> {
    let mut usage = HashMap::new();
    for id in containers.iter().filter_map(|c| c.image_id.as_ref()) {
        *usage.entry(id.clone()).or_insert(0) += 1;
    }
    usage
}

/// Converts one message of the pull stream.
pub fn pull_progress(info: CreateImageInfo) -> PullProgress {
    let detail = info.progress_detail.unwrap_or_default();
    PullProgress {
        layer: info.id.filter(|id| !id.is_empty()),
        status: info.status.unwrap_or_default(),
        current: detail.current.and_then(|n| u64::try_from(n).ok()),
        total: detail.total.and_then(|n| u64::try_from(n).ok()),
    }
}

/// An error in the pull stream, for example an unknown image, is an API error.
pub fn pull_error(error: Error) -> EngineError {
    match error {
        Error::DockerStreamError { error } => EngineError::Api(error),
        other => engine_error(other),
    }
}

#[cfg(test)]
mod tests;
