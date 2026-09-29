//! Copies files out of an extension image: a stopped container of the image, and
//! the archive endpoint (`GET /containers/{id}/archive`) that `docker cp` uses.

use std::io::Read;
use std::path::{Component, Path, PathBuf};

use bollard::Docker;
use bollard::models::ContainerCreateBody;
use bollard::query_parameters::{
    CreateContainerOptionsBuilder, DownloadFromContainerOptionsBuilder,
    RemoveContainerOptionsBuilder,
};
use captain_core::EngineError;
use futures::StreamExt;

use crate::mapping;

/// A container of the image that never starts. Remove it with [`remove`].
pub async fn create(docker: &Docker, image: &str, id: &str) -> Result<String, EngineError> {
    let name = format!("{id}-captain-copy");
    remove(docker, &name).await;
    let body = ContainerCreateBody {
        image: Some(image.to_string()),
        // Images built `FROM scratch` have no command, and create needs one.
        cmd: Some(vec!["captain-copy".into()]),
        ..ContainerCreateBody::default()
    };
    let options = CreateContainerOptionsBuilder::default().name(&name).build();
    docker
        .create_container(Some(options), body)
        .await
        .map(|created| created.id)
        .map_err(mapping::engine_error)
}

pub async fn remove(docker: &Docker, container: &str) {
    let options = RemoveContainerOptionsBuilder::default().force(true).build();
    docker.remove_container(container, Some(options)).await.ok();
}

/// The tar of `path` in `container`.
pub async fn archive(docker: &Docker, container: &str, path: &str) -> Result<Vec<u8>, EngineError> {
    let options = DownloadFromContainerOptionsBuilder::default()
        .path(path)
        .build();
    let mut stream = docker.download_from_container(container, Some(options));
    let mut bytes = Vec::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|error| {
            EngineError::Api(format!(
                "cannot copy {path} from the image: {}",
                mapping::engine_error(error)
            ))
        })?;
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}

/// The content of the one file in `tar`.
pub fn file_content(tar: &[u8]) -> Result<Vec<u8>, EngineError> {
    let mut archive = tar::Archive::new(tar);
    for entry in archive.entries().map_err(io_error)? {
        let mut entry = entry.map_err(io_error)?;
        if entry.header().entry_type().is_file() {
            let mut content = Vec::new();
            entry.read_to_end(&mut content).map_err(io_error)?;
            return Ok(content);
        }
    }
    Err(EngineError::Api("the archive has no file".into()))
}

/// Writes the regular files and folders of `tar` under `dest`. With `contents`, the
/// top folder of the archive is dropped, so a folder's contents land in `dest`.
/// Links are skipped, so no file can point outside `dest`. Returns the files made.
pub fn unpack(tar: &[u8], dest: &Path, contents: bool) -> Result<Vec<PathBuf>, EngineError> {
    std::fs::create_dir_all(dest).map_err(io_error)?;
    let mut archive = tar::Archive::new(tar);
    let mut written = Vec::new();
    for entry in archive.entries().map_err(io_error)? {
        let mut entry = entry.map_err(io_error)?;
        let kind = entry.header().entry_type();
        if !kind.is_file() && !kind.is_dir() {
            continue;
        }
        let path = entry.path().map_err(io_error)?.into_owned();
        let Some(relative) = safe_relative(&path, contents) else {
            continue;
        };
        let target = dest.join(relative);
        if kind.is_dir() {
            std::fs::create_dir_all(&target).map_err(io_error)?;
            continue;
        }
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(io_error)?;
        }
        let mut content = Vec::new();
        entry.read_to_end(&mut content).map_err(io_error)?;
        std::fs::write(&target, content).map_err(io_error)?;
        written.push(target);
    }
    Ok(written)
}

/// `path` without its first part when `strip` is set. `None` when nothing is left
/// or a part is not a plain name.
fn safe_relative(path: &Path, strip: bool) -> Option<PathBuf> {
    let mut parts = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => parts.push(part),
            Component::CurDir => {}
            _ => return None,
        }
    }
    let parts = if strip { parts.get(1..)? } else { &parts[..] };
    (!parts.is_empty()).then(|| parts.iter().collect())
}

/// Marks `file` executable, for host binaries.
pub fn make_executable(file: &Path) -> Result<(), EngineError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(file, std::fs::Permissions::from_mode(0o755)).map_err(io_error)?;
    }
    #[cfg(not(unix))]
    let _ = file;
    Ok(())
}

pub fn io_error(error: std::io::Error) -> EngineError {
    EngineError::Api(error.to_string())
}

#[cfg(test)]
mod tests;
