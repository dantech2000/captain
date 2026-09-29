//! Copies a volume: a helper in the source streams `/v` as a tar, and a helper in
//! the target unpacks it into a new volume with the same driver, options, and
//! labels. Then both sides are measured and compared. A switch-over copies into the
//! existing target volume again, after it empties it.

use std::collections::HashMap;

use bollard::Docker;
use bollard::models::{Volume, VolumeCreateRequest};
use bollard::query_parameters::{
    ListContainersOptionsBuilder, RemoveVolumeOptions, UploadToContainerOptionsBuilder,
};
use captain_core::EngineError;
use captain_core::migration::TransferEvent;
use futures::StreamExt;

use super::helper::{self, Mount};
use super::progress::{self, Events, Outcome};
use super::source::SourceEngine;
use super::verify::{MEASURE_SCRIPT, Measure};
use crate::mapping;

/// Empties a volume, hidden entries included.
const EMPTY_SCRIPT: &str = "find /v -mindepth 1 -delete";

/// What a volume copy does when the target already has the volume.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Existing {
    /// Skip it and leave it alone. Every normal copy does this.
    Skip,
    /// Empty it and copy again. Only a switch-over does this, once nothing in the
    /// source writes to the volume.
    Replace,
}

/// Copies the source volume `name` into the target volume `target_name`. A volume
/// that already exists in the target is skipped, unless `existing` is
/// [`Existing::Replace`]. If the copy into a new volume fails or is stopped, the
/// half-filled target volume is removed again.
pub async fn copy_volume(
    source: &SourceEngine,
    target: &Docker,
    (name, target_name): (&str, &str),
    existing: Existing,
    events: &Events,
) -> Result<Outcome, EngineError> {
    let exists = target.inspect_volume(target_name).await.is_ok();
    if exists && existing == Existing::Skip {
        return Ok(Outcome::Skipped(
            "A volume with this name is already in the target.".into(),
        ));
    }
    let volume = source.inspect_volume(name).await?;
    if let Some(note) = data_elsewhere(&volume) {
        if !exists {
            create(target, &volume, target_name).await?;
        }
        progress::send(events, TransferEvent::Note(note));
        return Ok(Outcome::Copied);
    }
    source.ensure_helper_image().await?;
    helper::ensure_image(target).await?;
    let measure = measure(source.run_script(name, MEASURE_SCRIPT).await?)?;
    if exists {
        empty(target, target_name).await?;
    } else {
        create(target, &volume, target_name).await?;
    }
    let copied = fill(source, target, name, target_name, measure, events).await;
    // A replaced volume stays: containers in the target may refer to it, and a
    // retry fills it again.
    if copied.is_err() && !exists {
        let options = RemoveVolumeOptions { force: true };
        if let Err(error) = target.remove_volume(target_name, Some(options)).await {
            tracing::warn!(%error, target_name, "cannot remove a half-copied volume");
        }
    }
    copied.map(|()| Outcome::Copied)
}

/// A note when the volume's data does not live in the engine: a `local` volume bound
/// to a host folder, or a volume from another driver. Captain then creates the
/// volume with the same options and copies no data, because that data is not inside
/// the engine and the target reaches the same place.
fn data_elsewhere(volume: &Volume) -> Option<String> {
    if volume.driver != "local" {
        return Some(format!(
            "The {} driver stores this volume, so Captain created it without copying data.",
            volume.driver
        ));
    }
    let device = volume.options.get("device")?;
    Some(format!(
        "This volume points at {device}, so Captain created it without copying data."
    ))
}

async fn create(target: &Docker, volume: &Volume, name: &str) -> Result<(), EngineError> {
    let request = VolumeCreateRequest {
        name: Some(name.into()),
        driver: Some(volume.driver.clone()),
        driver_opts: Some(volume.options.clone()),
        labels: Some(volume.labels.clone()),
        ..VolumeCreateRequest::default()
    };
    let created = target.create_volume(request).await;
    created.map(|_| ()).map_err(mapping::engine_error)
}

/// Empties the target volume `name` before a switch-over copies it again. It
/// refuses while a running container in the target uses the volume.
async fn empty(target: &Docker, name: &str) -> Result<(), EngineError> {
    let filters = HashMap::from([("volume", vec![name]), ("status", vec!["running"])]);
    let options = ListContainersOptionsBuilder::default()
        .filters(&filters)
        .build();
    let running = target.list_containers(Some(options)).await;
    let users: Vec<String> = running
        .map_err(mapping::engine_error)?
        .into_iter()
        .filter_map(|c| {
            c.names?
                .first()
                .map(|n| n.trim_start_matches('/').to_string())
        })
        .collect();
    if !users.is_empty() {
        return Err(EngineError::Api(format!(
            "{} runs in this engine and uses {name}. Roll back or stop it, then retry.",
            users.join(", ")
        )));
    }
    let mount = Mount {
        volume: name,
        read_only: false,
    };
    helper::run_script(target, "empty", Some(mount), EMPTY_SCRIPT).await?;
    Ok(())
}

fn measure(output: String) -> Result<Measure, EngineError> {
    Measure::parse(&output)
        .ok_or_else(|| EngineError::Api(format!("cannot measure the volume: {}", output.trim())))
}

/// Streams the data, then checks the copy.
async fn fill(
    source: &SourceEngine,
    target: &Docker,
    name: &str,
    target_name: &str,
    expected: Measure,
    events: &Events,
) -> Result<(), EngineError> {
    let mount = Mount {
        volume: target_name,
        read_only: false,
    };
    let writer = helper::create(target, "write", Some(mount), &["true"]).await?;
    let streamed = stream(source, target, name, &writer, expected.bytes, events).await;
    helper::remove(target, &writer).await;
    streamed?;
    if progress::is_cancelled(events) {
        return Err(progress::cancelled());
    }
    let mount = Mount {
        volume: target_name,
        read_only: true,
    };
    let output = helper::run_script(target, "check", Some(mount), MEASURE_SCRIPT).await?;
    expected
        .check(&measure(output)?)
        .map_err(EngineError::Api)?;
    progress::send(
        events,
        TransferEvent::Note(format!("Checked {} files and folders.", expected.entries)),
    );
    Ok(())
}

/// Pipes the source helper's tar of `/v` into the target helper at `/`, so the
/// entries land in its `/v`. The daemon keeps the owners from the tar headers.
async fn stream(
    source: &SourceEngine,
    target: &Docker,
    name: &str,
    writer: &str,
    total: u64,
    events: &Events,
) -> Result<(), EngineError> {
    let (reader, tar) = source.archive_volume(name).await?;
    let body = progress::counted(tar, total, events.clone())
        .map(|chunk| chunk.map(Into::into).map_err(std::io::Error::other));
    let options = UploadToContainerOptionsBuilder::default().path("/").build();
    let uploaded = target
        .upload_to_container(writer, Some(options), bollard::body_try_stream(body))
        .await;
    source.remove_helper(&reader).await;
    uploaded.map_err(mapping::engine_error)
}
