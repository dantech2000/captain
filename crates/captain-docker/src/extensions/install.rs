//! Prepare, install, remove, and list. See docs/features/0025-extensions.md.

use std::path::Path;
use std::pin::pin;
use std::time::{SystemTime, UNIX_EPOCH};

use bollard::Docker;
use bollard::query_parameters::{CreateImageOptionsBuilder, RemoveImageOptions};
use captain_core::EngineError;
use captain_core::extension::{
    Backend, ExtensionCandidate, ExtensionLabels, ExtensionMetadata, ExtensionPaths,
    InstalledExtension, MANIFEST_FILE, binary_name, extension_id, host_platform,
};
use captain_core::model::ImageReference;
use futures::StreamExt;

use super::manager::Context;
use super::{backend, files};
use crate::mapping;

/// Pulls the image if needed, then reads its labels and `metadata.json`.
pub async fn prepare(
    context: &Context,
    reference: &str,
) -> Result<ExtensionCandidate, EngineError> {
    let invalid = || EngineError::Api(format!("\"{reference}\" is not an image reference"));
    let parsed = ImageReference::parse(reference).ok_or_else(invalid)?;
    let image = parsed.to_string();
    let id = extension_id(&image).ok_or_else(invalid)?;
    let docker = &context.docker;
    if docker.inspect_image(&image).await.is_err() {
        pull(docker, &parsed).await?;
    }
    let inspect = docker
        .inspect_image(&image)
        .await
        .map_err(mapping::engine_error)?;
    let image_id = inspect.id.unwrap_or_default();
    let labels = inspect
        .config
        .and_then(|config| config.labels)
        .unwrap_or_default();
    let labels = ExtensionLabels::from_image(&labels).map_err(EngineError::Api)?;
    let container = files::create(docker, &image, &id).await?;
    let tar = files::archive(docker, &container, "/metadata.json").await;
    files::remove(docker, &container).await;
    let json = files::file_content(&tar?)?;
    let metadata =
        ExtensionMetadata::parse(&String::from_utf8_lossy(&json)).map_err(EngineError::Api)?;
    Ok(ExtensionCandidate {
        id,
        image,
        image_id,
        labels,
        metadata,
    })
}

pub(super) async fn pull(docker: &Docker, reference: &ImageReference) -> Result<(), EngineError> {
    let mut options = CreateImageOptionsBuilder::default().from_image(&reference.name);
    if !reference.tag.is_empty() {
        options = options.tag(&reference.tag);
    }
    let mut messages = pin!(docker.create_image(Some(options.build()), None, None));
    while let Some(message) = messages.next().await {
        message.map_err(mapping::pull_error)?;
    }
    Ok(())
}

/// Copies the files, starts the backend, and writes `extension.json` last. On a
/// failure, it removes the backend and the folder again.
pub async fn install(
    context: &Context,
    candidate: ExtensionCandidate,
) -> Result<InstalledExtension, EngineError> {
    let installed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let extension = InstalledExtension::new(candidate, installed);
    let dir = context.paths.dir(&extension.id);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).map_err(files::io_error)?;
    }
    let result = install_steps(context, &extension).await;
    if let Err(error) = &result {
        tracing::warn!(%error, id = %extension.id, "extension install failed; cleaning up");
        backend::down(context, &extension.id, true).await.ok();
        std::fs::remove_dir_all(&dir).ok();
    }
    result.map(|()| extension)
}

async fn install_steps(
    context: &Context,
    extension: &InstalledExtension,
) -> Result<(), EngineError> {
    let (docker, paths, id) = (&context.docker, &context.paths, &extension.id);
    let container = files::create(docker, &extension.image, id).await?;
    let copied = copy_files(docker, &container, paths, extension).await;
    files::remove(docker, &container).await;
    copied?;
    if let Some(backend) = extension.metadata.backend(&extension.image) {
        backend::up(context, extension, backend).await?;
    }
    std::fs::write(paths.manifest(id), extension.to_json()).map_err(files::io_error)
}

pub(super) async fn copy_files(
    docker: &Docker,
    container: &str,
    paths: &ExtensionPaths,
    extension: &InstalledExtension,
) -> Result<(), EngineError> {
    let id = &extension.id;
    std::fs::create_dir_all(paths.dir(id)).map_err(files::io_error)?;
    if let Some(tab) = extension.metadata.dashboard_tab() {
        let tar = files::archive(docker, container, &tab.root).await?;
        files::unpack(&tar, &paths.ui_dir(id), true)?;
    }
    for path in extension.metadata.host_binaries(host_platform()) {
        let tar = files::archive(docker, container, &path).await?;
        let bin = paths.bin_dir(id);
        files::unpack(&tar, &bin, false)?;
        files::make_executable(&bin.join(binary_name(&path)))?;
    }
    if let Some(Backend::Compose(file)) = extension.metadata.backend(&extension.image) {
        let folder = Path::new(&file)
            .parent()
            .map(|p| p.to_string_lossy().into_owned());
        let folder = folder
            .filter(|f| !f.is_empty() && f != "/")
            .unwrap_or_else(|| "/".into());
        let tar = files::archive(docker, container, &folder).await?;
        files::unpack(&tar, &paths.compose_dir(id), true)?;
    }
    Ok(())
}

/// Stops the backend and removes its volumes, deletes the folder, and removes the
/// image. A missing backend or image is not an error.
pub async fn remove(context: &Context, extension: &InstalledExtension) -> Result<(), EngineError> {
    if extension.metadata.vm.is_some() {
        backend::down(context, &extension.id, true).await?;
    }
    let dir = context.paths.dir(&extension.id);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).map_err(files::io_error)?;
    }
    let removed = context
        .docker
        .remove_image(&extension.image, None::<RemoveImageOptions>, None)
        .await;
    if let Err(error) = removed {
        tracing::warn!(%error, image = %extension.image, "cannot remove the extension image");
    }
    Ok(())
}

/// Every folder with a readable `extension.json`, sorted by title.
pub fn list(paths: &ExtensionPaths) -> Result<Vec<InstalledExtension>, EngineError> {
    let entries = match std::fs::read_dir(paths.root()) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(files::io_error(error)),
    };
    let mut extensions: Vec<InstalledExtension> = entries
        .filter_map(Result::ok)
        .filter_map(|entry| std::fs::read_to_string(entry.path().join(MANIFEST_FILE)).ok())
        .filter_map(|json| {
            InstalledExtension::from_json(&json)
                .inspect_err(|error| tracing::warn!(%error, "cannot read an extension manifest"))
                .ok()
        })
        .collect();
    extensions.sort_by_key(|extension| extension.title().to_lowercase());
    Ok(extensions)
}
