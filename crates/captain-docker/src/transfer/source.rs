//! The source engine, behind a read-only surface. The assistant must never change
//! the engine it copies from (ADR 0009), so this type offers only lists, inspects,
//! exports, and its own helper containers. It allows two writes, both opt-in: a
//! snapshot, which it names `captain-migrate/<container>:snapshot` and removes again,
//! and the switch-over's stop and roll-back start of containers the user confirmed.
//! Apart from its own helpers and snapshots, it removes nothing.

use std::collections::HashSet;
use std::pin::Pin;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use bollard::Docker;
use bollard::models::{
    ContainerConfig, ContainerInspectResponse, ContainerSummary, ImageSummary, Network,
    NetworkInspect, SystemDataUsageResponse, Volume,
};
use bollard::query_parameters::{
    CommitContainerOptionsBuilder, DataUsageOptionsBuilder, DownloadFromContainerOptionsBuilder,
    InspectContainerOptionsBuilder, ListContainersOptionsBuilder, ListImagesOptionsBuilder,
    ListNetworksOptions, ListVolumesOptions, RemoveImageOptions, StopContainerOptionsBuilder,
};
use captain_core::EngineError;
use captain_core::migration::HELPER_PREFIX;
use futures::{Stream, StreamExt};

use super::helper::{self, HELPER_IMAGE, Mount};
use crate::mapping;

/// A byte stream from the source engine. The chunks are `Vec<u8>`, because this
/// crate does not name bollard's `Bytes` type; the conversion does not copy.
pub type ByteStream = Pin<Box<dyn Stream<Item = Result<Vec<u8>, EngineError>> + Send>>;

/// The engine the assistant copies from. See the module docs for what it may do.
pub struct SourceEngine {
    docker: Docker,
    /// Helper containers this session created and has not removed.
    helpers: Mutex<HashSet<String>>,
    /// True if this session pulled the helper image, so it removes it at the end.
    pulled_helper: AtomicBool,
}

impl SourceEngine {
    pub fn new(docker: Docker) -> Self {
        Self {
            docker,
            helpers: Mutex::new(HashSet::new()),
            pulled_helper: AtomicBool::new(false),
        }
    }

    /// The engine's ID, to tell whether two endpoints reach one engine.
    pub async fn engine_id(&self) -> Result<Option<String>, EngineError> {
        let info = self.docker.info().await;
        info.map(|info| info.id).map_err(mapping::engine_error)
    }

    pub async fn list_volumes(&self) -> Result<Vec<Volume>, EngineError> {
        let list = self.docker.list_volumes(None::<ListVolumesOptions>).await;
        Ok(list
            .map_err(mapping::engine_error)?
            .volumes
            .unwrap_or_default())
    }

    /// Disk usage, for volume sizes.
    pub async fn disk_usage(&self) -> Result<SystemDataUsageResponse, EngineError> {
        let options = DataUsageOptionsBuilder::default().verbose(true).build();
        let usage = self.docker.df(Some(options)).await;
        usage.map_err(mapping::engine_error)
    }

    pub async fn inspect_volume(&self, name: &str) -> Result<Volume, EngineError> {
        let volume = self.docker.inspect_volume(name).await;
        volume.map_err(mapping::engine_error)
    }

    pub async fn list_images(&self) -> Result<Vec<ImageSummary>, EngineError> {
        let options = ListImagesOptionsBuilder::default().all(false).build();
        let images = self.docker.list_images(Some(options)).await;
        images.map_err(mapping::engine_error)
    }

    /// Every container, with the size of its own filesystem changes.
    pub async fn list_containers(&self) -> Result<Vec<ContainerSummary>, EngineError> {
        let options = ListContainersOptionsBuilder::default()
            .all(true)
            .size(true)
            .build();
        let containers = self.docker.list_containers(Some(options)).await;
        containers.map_err(mapping::engine_error)
    }

    pub async fn inspect_container(
        &self,
        id: &str,
    ) -> Result<ContainerInspectResponse, EngineError> {
        let options = InspectContainerOptionsBuilder::default().size(true).build();
        let inspect = self.docker.inspect_container(id, Some(options)).await;
        inspect.map_err(mapping::engine_error)
    }

    pub async fn list_networks(&self) -> Result<Vec<Network>, EngineError> {
        let networks = self.docker.list_networks(None::<ListNetworksOptions>).await;
        networks.map_err(mapping::engine_error)
    }

    pub async fn inspect_network(&self, name: &str) -> Result<NetworkInspect, EngineError> {
        let network = self.docker.inspect_network(name, None).await;
        network.map_err(mapping::engine_error)
    }

    /// `GET /images/get` for `names`. Save by tag, so the tags travel with the image.
    pub fn export_images(&self, names: &[String]) -> ByteStream {
        let names: Vec<&str> = names.iter().map(String::as_str).collect();
        chunks(self.docker.export_images(&names))
    }

    /// Pulls the helper image if the source lacks it, and remembers that it did.
    pub async fn ensure_helper_image(&self) -> Result<(), EngineError> {
        if helper::ensure_image(&self.docker).await? {
            self.pulled_helper.store(true, Ordering::Relaxed);
        }
        Ok(())
    }

    /// Runs a script in a helper that mounts `volume` read-only.
    pub async fn run_script(&self, volume: &str, script: &str) -> Result<String, EngineError> {
        let mount = Mount {
            volume,
            read_only: true,
        };
        helper::run_script(&self.docker, "read", Some(mount), script).await
    }

    /// Creates a helper that mounts `volume` read-only at `/v`, and streams `/v` as a
    /// tar. Remove the helper with [`Self::remove_helper`] when the stream ends.
    pub async fn archive_volume(&self, volume: &str) -> Result<(String, ByteStream), EngineError> {
        let mount = Mount {
            volume,
            read_only: true,
        };
        let id = helper::create(&self.docker, "read", Some(mount), &["true"]).await?;
        self.helpers.lock().expect("helpers").insert(id.clone());
        let options = DownloadFromContainerOptionsBuilder::default()
            .path("/v")
            .build();
        let stream = self.docker.download_from_container(&id, Some(options));
        Ok((id, chunks(stream)))
    }

    /// Removes a helper this session created. Other containers are refused.
    pub async fn remove_helper(&self, id: &str) {
        if self.helpers.lock().expect("helpers").remove(id) {
            helper::remove(&self.docker, id).await;
        }
    }

    /// Commits `container` to `captain-migrate/<name>:snapshot` without pausing it,
    /// and returns the reference. This is the one write to the source, and only on
    /// the user's request.
    pub async fn snapshot(&self, container: &str, name: &str) -> Result<String, EngineError> {
        let repo = snapshot_repo(name);
        let options = CommitContainerOptionsBuilder::default()
            .container(container)
            .repo(&repo)
            .tag("snapshot")
            .comment("Captain migration snapshot")
            .pause(false)
            .build();
        self.docker
            .commit_container(options, ContainerConfig::default())
            .await
            .map_err(mapping::engine_error)?;
        Ok(format!("{repo}:snapshot"))
    }

    /// Removes a snapshot image. Only `captain-migrate/` references are accepted.
    pub async fn remove_snapshot(&self, reference: &str) -> Result<(), EngineError> {
        if !reference.starts_with(&format!("{HELPER_PREFIX}/")) {
            return Err(EngineError::Api(format!(
                "Captain removes only its own snapshots, not {reference}"
            )));
        }
        let removed = self
            .docker
            .remove_image(reference, None::<RemoveImageOptions>, None)
            .await;
        removed.map(|_| ()).map_err(mapping::engine_error)
    }

    /// Stops `container` and waits up to `timeout` seconds before the engine kills
    /// it. This is the switch-over's write to the source, only for containers the
    /// user confirmed. It never removes the container.
    pub async fn stop_container(&self, container: &str, timeout: i32) -> Result<(), EngineError> {
        let options = StopContainerOptionsBuilder::default().t(timeout).build();
        let stopped = self.docker.stop_container(container, Some(options)).await;
        stopped.map_err(mapping::engine_error)
    }

    /// Starts `container` again, to roll back a switch-over.
    pub async fn start_container(&self, container: &str) -> Result<(), EngineError> {
        let started = self.docker.start_container(container, None).await;
        started.map_err(mapping::engine_error)
    }

    /// Removes this session's helpers, any helper left from an earlier session, and
    /// the helper image if this session pulled it.
    pub async fn clean_up(&self) {
        let mine: Vec<String> = self.helpers.lock().expect("helpers").drain().collect();
        let left = helper::leftovers(&self.docker).await.unwrap_or_default();
        for id in mine.iter().chain(&left) {
            helper::remove(&self.docker, id).await;
        }
        if self.pulled_helper.swap(false, Ordering::Relaxed) {
            let removed = self
                .docker
                .remove_image(HELPER_IMAGE, None::<RemoveImageOptions>, None)
                .await;
            if let Err(error) = removed {
                tracing::warn!(%error, "cannot remove the helper image from the source");
            }
        }
    }
}

fn chunks<S, B>(stream: S) -> ByteStream
where
    S: Stream<Item = Result<B, bollard::errors::Error>> + Send + 'static,
    B: Into<Vec<u8>>,
{
    Box::pin(stream.map(|chunk| chunk.map(Into::into).map_err(mapping::engine_error)))
}

/// `captain-migrate/<name>`, lowercased, with characters an image name cannot hold
/// replaced by `-`.
pub fn snapshot_repo(name: &str) -> String {
    let clean: String = name
        .trim_start_matches('/')
        .chars()
        .map(|c| match c.to_ascii_lowercase() {
            c @ ('a'..='z' | '0'..='9' | '.' | '_' | '-') => c,
            _ => '-',
        })
        .collect();
    format!("{HELPER_PREFIX}/{}", clean.trim_matches(['.', '_', '-']))
}

#[cfg(test)]
mod tests;
