//! Copies an image: `GET /images/get` from the source streams straight into
//! `POST /images/load` on the target, tags included.

use bollard::Docker;
use bollard::query_parameters::ImportImageOptionsBuilder;
use captain_core::EngineError;
use futures::StreamExt;

use super::progress::{self, Events, Outcome};
use super::source::SourceEngine;
use crate::mapping;

/// Copies the image `id` with its `tags`. It saves by tag, because a save by ID
/// carries no tags; an untagged image goes by ID. An image the target already has
/// with every tag is skipped, since a load is not atomic (moby/moby#48591).
pub async fn copy_image(
    source: &SourceEngine,
    target: &Docker,
    id: &str,
    tags: &[String],
    size: u64,
    events: &Events,
) -> Result<Outcome, EngineError> {
    if has_image(target, id, tags).await {
        return Ok(Outcome::Skipped(
            "The target already has this image.".into(),
        ));
    }
    let names = if tags.is_empty() {
        vec![id.to_string()]
    } else {
        tags.to_vec()
    };
    let export = progress::counted(source.export_images(&names), size, events.clone())
        .map(|chunk| chunk.map(Into::into));
    // Leave `platform` unset: bollard cannot encode it (a list), and the engine
    // then loads what the tar holds.
    let options = ImportImageOptionsBuilder::default().quiet(true).build();
    let mut load = target.import_image_stream(options, export, None);
    while let Some(message) = load.next().await {
        message.map_err(|error| load_error(mapping::engine_error(error)))?;
    }
    if progress::is_cancelled(events) {
        return Err(progress::cancelled());
    }
    Ok(Outcome::Copied)
}

/// True if the target has `id` under every one of `tags`.
async fn has_image(target: &Docker, id: &str, tags: &[String]) -> bool {
    let Ok(image) = target.inspect_image(id).await else {
        return false;
    };
    let have = image.repo_tags.unwrap_or_default();
    tags.iter().all(|tag| have.contains(tag))
}

/// Adds a hint to the containerd store's error for a partly pulled image.
fn load_error(error: EngineError) -> EngineError {
    let text = error.to_string();
    if text.contains("not found") && text.contains("content") {
        return EngineError::Api(format!(
            "{text}. The source has only some platforms of this image. Pull it again in the source for one platform, then retry."
        ));
    }
    error
}
