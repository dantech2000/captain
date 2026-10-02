//! Images that an install or update pulled, which Captain owns and removes again
//! when the user cancels or the step fails. See docs/features/0025-extensions.md.

use bollard::Docker;
use bollard::query_parameters::RemoveImageOptions;
use captain_core::EngineError;
use captain_core::extension::ExtensionCandidate;

use super::manager::Context;
use crate::mapping;

/// True only when the engine answers that `image` does not exist. Then a pull
/// brings in an image that Captain owns and may remove again.
pub async fn absent(docker: &Docker, image: &str) -> Result<bool, EngineError> {
    Ok(mapping::found(docker.inspect_image(image).await)?.is_none())
}

/// Removes an image that an install pulled and no longer needs, by its immutable ID
/// and without force: a tag can move to another image meanwhile, and the engine
/// keeps an image that another tag still names. A failure only logs.
pub async fn remove_image(docker: &Docker, image_id: &str) {
    if image_id.is_empty() {
        return;
    }
    let removed = docker
        .remove_image(image_id, None::<RemoveImageOptions>, None)
        .await;
    match removed {
        Ok(_) => tracing::info!(%image_id, "removed the pulled extension image"),
        Err(error) => {
            tracing::info!(%error, %image_id, "cannot remove the pulled extension image");
        }
    }
}

/// Removes the image of a candidate the user did not install, if Captain pulled it.
pub async fn discard(context: &Context, candidate: &ExtensionCandidate) {
    if candidate.pulled {
        remove_image(&context.docker, &candidate.image_id).await;
    }
}
