//! Converts `inspect` and `history` results for the image inspector.

use bollard::errors::Error;
use bollard::models::{ImageHistoryResponseItem, ImageInspect};
use captain_core::EngineError;
use captain_core::model::{EnvVar, ExposedPort, ImageConfig, ImageDetail, ImageLayer};

use super::{UNTAGGED, engine_error};

/// Converts an image `inspect` result.
pub fn image_detail(inspect: ImageInspect) -> ImageDetail {
    let config = inspect.config.unwrap_or_default();
    let mut exposed_ports: Vec<ExposedPort> = config
        .exposed_ports
        .unwrap_or_default()
        .iter()
        .filter_map(|port| ExposedPort::parse(port))
        .collect();
    exposed_ports.sort();
    let mut labels: Vec<(String, String)> = config.labels.unwrap_or_default().into_iter().collect();
    labels.sort();

    ImageDetail {
        id: inspect.id.unwrap_or_default(),
        repo_tags: inspect
            .repo_tags
            .unwrap_or_default()
            .into_iter()
            .filter(|tag| tag != UNTAGGED)
            .collect(),
        repo_digests: inspect.repo_digests.unwrap_or_default(),
        created: inspect.created.unwrap_or_default(),
        architecture: inspect.architecture.unwrap_or_default(),
        variant: inspect.variant.unwrap_or_default(),
        os: inspect.os.unwrap_or_default(),
        size: inspect
            .size
            .and_then(|bytes| u64::try_from(bytes).ok())
            .unwrap_or(0),
        config: ImageConfig {
            entrypoint: config.entrypoint.unwrap_or_default(),
            cmd: config.cmd.unwrap_or_default(),
            env: config
                .env
                .unwrap_or_default()
                .iter()
                .map(|entry| EnvVar::parse(entry))
                .collect(),
            exposed_ports,
            working_dir: config.working_dir.unwrap_or_default(),
            user: config.user.unwrap_or_default(),
            labels,
        },
    }
}

/// Converts a history result. The engine answers `null` for an image with no history,
/// for example an imported one, and bollard fails to read that. That is no history.
pub fn image_history(
    result: Result<Vec<ImageHistoryResponseItem>, Error>,
) -> Result<Vec<ImageLayer>, EngineError> {
    match result {
        Ok(items) => Ok(items.into_iter().map(image_layer).collect()),
        Err(Error::JsonDataError { message, .. }) if message.starts_with("invalid type: null") => {
            Ok(Vec::new())
        }
        Err(error) => Err(engine_error(error)),
    }
}

/// Converts one step of an image's history.
pub fn image_layer(item: ImageHistoryResponseItem) -> ImageLayer {
    ImageLayer {
        id: item.id,
        created: item.created,
        created_by: item.created_by,
        size: u64::try_from(item.size).unwrap_or(0),
    }
}

#[cfg(test)]
mod tests;
