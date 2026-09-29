//! Recreates a container in the target from its inspect data, optionally from a
//! snapshot of its own filesystem.

mod spec;

use bollard::Docker;
use bollard::models::{ContainerInspectResponse, NetworkConnectRequest};
use bollard::query_parameters::CreateContainerOptionsBuilder;
use captain_core::EngineError;
use captain_core::migration::TransferEvent;

use super::image::copy_image;
use super::owner;
use super::progress::{self, Events, Outcome};
use super::source::SourceEngine;
use crate::mapping;

/// Recreates the source container `id` (an ID or name) in the target, under
/// `target_name` or else its own name. It starts only if the source container runs.
/// One with that name in the target is skipped. With `snapshot`, the copy runs a
/// commit of the container.
pub async fn copy_container(
    source: &SourceEngine,
    target: &Docker,
    id: &str,
    target_name: Option<&str>,
    snapshot: bool,
    events: &Events,
) -> Result<Outcome, EngineError> {
    let inspect = source.inspect_container(id).await?;
    let name = inspect
        .name
        .clone()
        .unwrap_or_default()
        .trim_start_matches('/')
        .to_string();
    let target_name = target_name.unwrap_or(&name);
    if target.inspect_container(target_name, None).await.is_ok() {
        let reason = "A container with this name is already in the target.";
        return Ok(Outcome::Skipped(reason.into()));
    }
    let origin = owner::origin(source).await?;
    let copy = Target {
        inspect: &inspect,
        name: target_name,
        origin: &origin,
    };
    if !snapshot {
        let configured = inspect.config.as_ref().and_then(|c| c.image.clone());
        let image = target_image(target, configured, inspect.image.clone()).await?;
        return create(target, &copy, &image, events).await;
    }
    let size = inspect.size_rw.and_then(|s| u64::try_from(s).ok());
    let reference = source.snapshot(id, &name).await?;
    let tags = [reference.clone()];
    let copied = copy_image(source, target, &reference, &tags, size.unwrap_or(0), events).await;
    let created = match copied {
        Ok(_) => create(target, &copy, &reference, events).await,
        Err(error) => Err(error),
    };
    // The snapshot is the one thing Captain writes to the source. It goes again
    // whether or not the copy worked.
    if let Err(error) = source.remove_snapshot(&reference).await {
        tracing::warn!(%error, reference, "cannot remove the snapshot from the source");
        let note = format!("Captain could not remove {reference} from the old engine: {error}");
        progress::send(events, TransferEvent::Note(note));
    }
    created
}

/// The container to create in the target.
struct Target<'a> {
    inspect: &'a ContainerInspectResponse,
    name: &'a str,
    /// The source engine, for the mark from [`owner`].
    origin: &'a str,
}

/// Creates the copy from `image`, marks it, joins its networks, and starts it if
/// the original runs.
async fn create(
    target: &Docker,
    copy: &Target<'_>,
    image: &str,
    events: &Events,
) -> Result<Outcome, EngineError> {
    let mut recreate = spec::recreate(copy.inspect, image);
    recreate.body.labels = Some(owner::mark(recreate.body.labels.take(), copy.origin));
    let options = CreateContainerOptionsBuilder::default()
        .name(copy.name)
        .build();
    let created = target
        .create_container(Some(options), recreate.body)
        .await
        .map_err(mapping::engine_error)?;
    for (network, endpoint) in recreate.extra_networks {
        let request = NetworkConnectRequest {
            container: created.id.clone(),
            endpoint_config: Some(endpoint),
        };
        let connected = target.connect_network(&network, request).await;
        connected.map_err(mapping::engine_error)?;
    }
    if recreate.start {
        let started = target.start_container(&created.id, None).await;
        started.map_err(mapping::engine_error)?;
    } else {
        let note = "Created, not started, like the original.";
        progress::send(events, TransferEvent::Note(note.into()));
    }
    Ok(Outcome::Copied)
}

/// The image reference to create the copy from: the one the container was created
/// with, or else its image ID. The image must already be in the target.
async fn target_image(
    target: &Docker,
    configured: Option<String>,
    id: Option<String>,
) -> Result<String, EngineError> {
    for reference in configured.iter().chain(id.iter()) {
        if target.inspect_image(reference).await.is_ok() {
            return Ok(reference.clone());
        }
    }
    let shown = configured.or(id).unwrap_or_default();
    Err(EngineError::Api(format!(
        "The image {shown} is not in the target. Select it in the plan, then retry."
    )))
}
