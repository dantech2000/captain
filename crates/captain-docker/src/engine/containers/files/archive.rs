//! Reads `GET /containers/{id}/archive`, the tar that `docker cp` uses.

use std::fs::File;
use std::io::Write;
use std::path::Path;

use bollard::Docker;
use bollard::query_parameters::DownloadFromContainerOptionsBuilder;
use captain_core::EngineError;
use futures::StreamExt;

use crate::mapping;

/// What stopped a capped read.
pub enum Collected {
    /// The whole tar.
    All(Vec<u8>),
    /// The first `cap` bytes or more; the tar is longer.
    Capped(Vec<u8>),
}

/// Reads the tar of `path`, but stops after `cap` bytes.
pub async fn collect(
    docker: &Docker,
    id: &str,
    path: &str,
    cap: usize,
) -> Result<Collected, EngineError> {
    let options = DownloadFromContainerOptionsBuilder::default()
        .path(path)
        .build();
    let mut stream = docker.download_from_container(id, Some(options));
    let mut bytes = Vec::new();
    while let Some(chunk) = stream.next().await {
        bytes.extend_from_slice(&chunk.map_err(mapping::engine_error)?);
        if bytes.len() >= cap {
            return Ok(Collected::Capped(bytes));
        }
    }
    Ok(Collected::All(bytes))
}

/// Streams the tar of `path` into the file `target`.
pub async fn write_to(
    docker: &Docker,
    id: &str,
    path: &str,
    target: &Path,
) -> Result<(), EngineError> {
    let options = DownloadFromContainerOptionsBuilder::default()
        .path(path)
        .build();
    let mut stream = docker.download_from_container(id, Some(options));
    let mut file = File::create(target).map_err(io_error)?;
    while let Some(chunk) = stream.next().await {
        file.write_all(&chunk.map_err(mapping::engine_error)?)
            .map_err(io_error)?;
    }
    file.flush().map_err(io_error)
}

pub fn io_error(error: std::io::Error) -> EngineError {
    EngineError::Api(error.to_string())
}
