//! Check for an update and apply it. See docs/features/0025-extensions.md.

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use bollard::query_parameters::RemoveImageOptions;
use captain_core::EngineError;
use captain_core::extension::{
    ExtensionCandidate, ExtensionPaths, ExtensionUpdate, InstalledExtension, UpdateCheck,
    update_reference,
};
use captain_core::model::ImageReference;

use super::manager::Context;
use super::swap::Swap;
use super::{backend, files, install};

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
/// volumes, moves the old files to a backup folder and the new ones in, and starts
/// the new backend. The backup goes only after that start; on a failure, Captain
/// puts the old files back and starts the old backend again.
pub async fn apply(
    context: &Context,
    extension: InstalledExtension,
    candidate: ExtensionCandidate,
) -> Result<InstalledExtension, EngineError> {
    let installed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let mut new = InstalledExtension::new(candidate, installed);
    // The ID stays, so an older ID keeps its folder, backend, and page data.
    new.id = extension.id.clone();
    new.engine = context.engine.clone();
    let root = context.paths.root();
    let backup = root.join(".backup").join(&new.id);
    if backup.exists() {
        return Err(EngineError::Api(format!(
            "An earlier update of {} did not finish. Its previous files are in {}.",
            extension.title(),
            backup.display()
        )));
    }
    let staging = ExtensionPaths::new(root.join(".update"));
    std::fs::remove_dir_all(staging.dir(&new.id)).ok();
    let result = stage_and_switch(context, &extension, &new, &staging, backup).await;
    std::fs::remove_dir_all(staging.dir(&new.id)).ok();
    std::fs::remove_dir(staging.root()).ok();
    result?;
    remove_old_image(context, &extension, &new).await;
    Ok(new)
}

async fn stage_and_switch(
    context: &Context,
    old: &InstalledExtension,
    new: &InstalledExtension,
    staging: &ExtensionPaths,
    backup: PathBuf,
) -> Result<(), EngineError> {
    let docker = &context.docker;
    let container = files::create(docker, new.pinned_image(), &new.id).await?;
    let copied = install::copy_files(docker, &container, staging, new).await;
    files::remove(docker, &container).await;
    copied?;
    let live = context.paths.dir(&new.id);
    let mut swap = Swap::new(live, staging.dir(&new.id), backup);
    match switch(context, old, new, &mut swap).await {
        Ok(()) => {
            swap.commit();
            Ok(())
        }
        Err(error) => Err(roll_back(context, old, new, &swap, error).await),
    }
}

async fn switch(
    context: &Context,
    old: &InstalledExtension,
    new: &InstalledExtension,
    swap: &mut Swap,
) -> Result<(), EngineError> {
    if old.metadata.vm.is_some() {
        backend::down(context, &old.id, false).await?;
    }
    swap.replace(&new.to_json())?;
    if let Some(backend) = new.metadata.backend(new.pinned_image()) {
        backend::up(context, new, backend).await?;
    }
    Ok(())
}

/// Stops what the new version started, puts the old files back, and starts the old
/// backend. Returns `error`, with what went wrong on the way back.
async fn roll_back(
    context: &Context,
    old: &InstalledExtension,
    new: &InstalledExtension,
    swap: &Swap,
    error: EngineError,
) -> EngineError {
    tracing::warn!(%error, id = %old.id, "extension update failed; restoring the old version");
    if new.metadata.vm.is_some() {
        backend::down(context, &new.id, false).await.ok();
    }
    let restored = match (swap.roll_back(), old.metadata.backend(old.pinned_image())) {
        (Ok(()), Some(backend)) => backend::up(context, old, backend).await,
        (result, _) => result,
    };
    match restored {
        Ok(()) => error,
        Err(failed) => EngineError::Api(format!(
            "{}. Captain could not restore the old version: {}",
            message(error),
            message(failed)
        )),
    }
}

fn message(error: EngineError) -> String {
    match error {
        EngineError::Api(message) | EngineError::Unreachable(message) => message,
    }
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
