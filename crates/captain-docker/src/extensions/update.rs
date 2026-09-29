//! Check for an update and apply it. See docs/features/0025-extensions.md.

use std::time::{SystemTime, UNIX_EPOCH};

use bollard::query_parameters::RemoveImageOptions;
use captain_core::EngineError;
use captain_core::extension::{
    ExtensionCandidate, ExtensionPaths, ExtensionUpdate, InstalledExtension, UpdateCheck,
    update_reference,
};
use captain_core::model::ImageReference;

use super::manager::Context;
use super::{backend, files, install};

/// The folders an update replaces. Everything else in the extension's folder stays.
const REPLACED: [&str; 3] = ["ui", "bin", "compose"];

/// Pulls the repository with `tag` and compares the image with the installed one.
pub async fn check(
    context: &Context,
    extension: InstalledExtension,
    tag: &str,
) -> Result<UpdateCheck, EngineError> {
    let invalid = || EngineError::Api(format!("\"{tag}\" is not a valid tag"));
    let reference = update_reference(&extension.image, tag).ok_or_else(invalid)?;
    let docker = &context.docker;
    // Read the installed ID before the pull, which may move its tag.
    let installed_id = match extension.image_id.as_str() {
        "" => docker
            .inspect_image(&extension.image)
            .await
            .ok()
            .and_then(|image| image.id),
        id => Some(id.to_string()),
    };
    let parsed = ImageReference::parse(&reference).ok_or_else(invalid)?;
    if let Err(error) = install::pull(docker, &parsed).await {
        if docker.inspect_image(&reference).await.is_err() {
            return Err(error);
        }
        tracing::info!(%error, %reference, "cannot pull the update; using the engine's copy");
    }
    let candidate = install::prepare(context, &reference).await?;
    if installed_id.as_deref() == Some(candidate.image_id.as_str()) {
        return Ok(UpdateCheck::UpToDate { image: reference });
    }
    Ok(UpdateCheck::Available(Box::new(ExtensionUpdate {
        extension,
        candidate,
    })))
}

/// Copies the new files to a staging folder, stops the old backend without its
/// volumes, swaps the folders, and starts the new backend.
pub async fn apply(
    context: &Context,
    extension: InstalledExtension,
    candidate: ExtensionCandidate,
) -> Result<InstalledExtension, EngineError> {
    let installed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let new = InstalledExtension::new(candidate, installed);
    let staging_root = context.paths.root().join(".update");
    let staging = ExtensionPaths::new(staging_root.clone());
    std::fs::remove_dir_all(staging.dir(&new.id)).ok();
    let result = swap(context, &extension, &new, &staging).await;
    std::fs::remove_dir_all(&staging_root).ok();
    result?;

    let (paths, id) = (&context.paths, &new.id);
    std::fs::write(paths.manifest(id), new.to_json()).map_err(files::io_error)?;
    if let Some(backend) = new.metadata.backend(&new.image) {
        backend::up(context, &new, backend).await?;
    }
    remove_old_image(context, &extension, &new).await;
    Ok(new)
}

async fn swap(
    context: &Context,
    old: &InstalledExtension,
    new: &InstalledExtension,
    staging: &ExtensionPaths,
) -> Result<(), EngineError> {
    let docker = &context.docker;
    let container = files::create(docker, &new.image, &new.id).await?;
    let copied = install::copy_files(docker, &container, staging, new).await;
    files::remove(docker, &container).await;
    copied?;
    if old.metadata.vm.is_some() {
        backend::down(context, &old.id, false).await?;
    }
    let (from, to) = (staging.dir(&new.id), context.paths.dir(&new.id));
    for part in REPLACED {
        let target = to.join(part);
        if target.exists() {
            std::fs::remove_dir_all(&target).map_err(files::io_error)?;
        }
        if from.join(part).exists() {
            std::fs::rename(from.join(part), &target).map_err(files::io_error)?;
        }
    }
    Ok(())
}

/// Removes the old image when the update replaced it. Its old tag, if the new image
/// has another, or else its ID, since the pull moved the tag. A failure only logs.
async fn remove_old_image(context: &Context, old: &InstalledExtension, new: &InstalledExtension) {
    if old.image_id == new.image_id {
        return;
    }
    let target = if old.image != new.image {
        &old.image
    } else if !old.image_id.is_empty() {
        &old.image_id
    } else {
        return;
    };
    let removed = context
        .docker
        .remove_image(target, None::<RemoveImageOptions>, None)
        .await;
    if let Err(error) = removed {
        tracing::info!(%error, image = %target, "cannot remove the old extension image");
    }
}
