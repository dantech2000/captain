//! Copies a volume: a helper in the source streams `/v` as a tar, and a helper in
//! the target unpacks it into a new volume with the same driver, options, and
//! labels. Then both sides are measured and compared.

use bollard::Docker;
use bollard::models::{Volume, VolumeCreateRequest};
use bollard::query_parameters::{RemoveVolumeOptions, UploadToContainerOptionsBuilder};
use captain_core::EngineError;
use captain_core::migration::TransferEvent;
use futures::StreamExt;

use super::helper::{self, Mount};
use super::progress::{self, Events, Outcome};
use super::source::SourceEngine;
use super::verify::{MEASURE_SCRIPT, Measure};
use crate::mapping;

/// Copies the source volume `name` into a new target volume `target_name`. A volume
/// that already exists in the target is skipped, never overwritten. If the copy
/// fails or is stopped, the half-filled target volume is removed again.
pub async fn copy_volume(
    source: &SourceEngine,
    target: &Docker,
    name: &str,
    target_name: &str,
    events: &Events,
) -> Result<Outcome, EngineError> {
    if target.inspect_volume(target_name).await.is_ok() {
        return Ok(Outcome::Skipped(
            "A volume with this name is already in the target.".into(),
        ));
    }
    let volume = source.inspect_volume(name).await?;
    if let Some(note) = data_elsewhere(&volume) {
        create(target, &volume, target_name).await?;
        progress::send(events, TransferEvent::Note(note));
        return Ok(Outcome::Copied);
    }
    source.ensure_helper_image().await?;
    helper::ensure_image(target).await?;
    let measure = measure(source.run_script(name, MEASURE_SCRIPT).await?)?;
    create(target, &volume, target_name).await?;
    let copied = fill(source, target, name, target_name, measure, events).await;
    if copied.is_err() {
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
