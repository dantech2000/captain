//! Copies an extension's files out of its image: the UI, the host binaries for
//! this platform, and the Compose file with the files next to it.

use std::path::Path;

use bollard::Docker;
use captain_core::EngineError;
use captain_core::extension::{
    Backend, ExtensionPaths, InstalledExtension, binary_name, compose_references, host_platform,
};

use super::files;

pub async fn copy_files(
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
        copy_compose(docker, container, &file, &paths.compose_dir(id)).await?;
    }
    Ok(())
}

/// Copies the Compose file's folder, as Rancher Desktop does. A Compose file at
/// the image root would copy the whole image, so there Captain copies the file and
/// the files it names by a relative path.
async fn copy_compose(
    docker: &Docker,
    container: &str,
    file: &str,
    dest: &Path,
) -> Result<(), EngineError> {
    let file = format!("/{}", file.trim_start_matches("./").trim_start_matches('/'));
    let folder = Path::new(&file)
        .parent()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();
    if folder != "/" && !folder.is_empty() {
        let tar = files::archive(docker, container, &folder).await?;
        files::unpack(&tar, dest, true)?;
        return Ok(());
    }
    let tar = files::archive(docker, container, &file).await?;
    let written = files::unpack(&tar, dest, false)?;
    let yaml = written
        .first()
        .and_then(|path| std::fs::read_to_string(path).ok())
        .unwrap_or_default();
    for reference in compose_references(&yaml) {
        // A missing file is Compose's to report; `.env` is optional.
        let Ok(tar) = files::archive(docker, container, &format!("/{reference}")).await else {
            tracing::debug!(%reference, "the Compose file names a path the image does not have");
            continue;
        };
        let parent = Path::new(&reference).parent().unwrap_or(Path::new(""));
        files::unpack(&tar, &dest.join(parent), false)?;
    }
    Ok(())
}
