//! Volumes. Every call runs on the private tokio runtime.

use std::collections::HashMap;

use bollard::models::VolumeCreateRequest;
use bollard::query_parameters::{
    DataUsageOptionsBuilder, ListContainersOptionsBuilder, ListVolumesOptions,
    PruneVolumesOptionsBuilder, RemoveVolumeOptions,
};
use captain_core::model::{Volume, VolumePrune, VolumeUser};
use captain_core::{EngineFuture, VolumeApi};

use super::DockerEngine;
use crate::{mapping, runtime};

impl VolumeApi for DockerEngine {
    fn list_volumes(&self) -> EngineFuture<Vec<Volume>> {
        let docker = self.docker.clone();
        runtime::spawn(self.runtime.handle(), async move {
            // Bollard cannot encode the `type` filter, so df reports every kind.
            // `verbose` makes API 1.52+ list each volume with its usage.
            let options = DataUsageOptionsBuilder::default().verbose(true).build();
            let (list, usage) = futures::join!(
                docker.list_volumes(None::<ListVolumesOptions>),
                docker.df(Some(options))
            );
            let list = list.map_err(mapping::engine_error)?;
            // Sizes are extra. Without them the list still shows, with unknown usage.
            let usage = usage.map(mapping::volume_usage).unwrap_or_else(|error| {
                tracing::warn!(%error, "could not read volume disk usage");
                mapping::VolumeUsage::default()
            });
            Ok(list
                .volumes
                .unwrap_or_default()
                .into_iter()
                .map(|volume| mapping::volume(volume, &usage))
                .collect())
        })
    }

    fn create_volume(&self, name: &str) -> EngineFuture<()> {
        let docker = self.docker.clone();
        let request = VolumeCreateRequest {
            name: Some(name.to_string()),
            ..VolumeCreateRequest::default()
        };
        runtime::spawn(self.runtime.handle(), async move {
            docker
                .create_volume(request)
                .await
                .map(|_| ())
                .map_err(mapping::engine_error)
        })
    }

    fn remove_volume(&self, name: &str) -> EngineFuture<()> {
        let docker = self.docker.clone();
        let name = name.to_string();
        runtime::spawn(self.runtime.handle(), async move {
            docker
                .remove_volume(&name, None::<RemoveVolumeOptions>)
                .await
                .map_err(mapping::engine_error)
        })
    }

    fn volume_users(&self, name: &str) -> EngineFuture<Vec<VolumeUser>> {
        let docker = self.docker.clone();
        let name = name.to_string();
        runtime::spawn(self.runtime.handle(), async move {
            let filters = HashMap::from([("volume", vec![name.as_str()])]);
            let options = ListContainersOptionsBuilder::default()
                .all(true)
                .filters(&filters)
                .build();
            let containers = docker
                .list_containers(Some(options))
                .await
                .map_err(mapping::engine_error)?;
            Ok(mapping::volume_users(containers, &name))
        })
    }

    fn prune_unused_volumes(&self, all: bool, label: Option<&str>) -> EngineFuture<VolumePrune> {
        let docker = self.docker.clone();
        let filters = mapping::volume_prune_filters(all, label);
        runtime::spawn(self.runtime.handle(), async move {
            let options = PruneVolumesOptionsBuilder::default()
                .filters(&filters)
                .build();
            docker
                .prune_volumes(Some(options))
                .await
                .map(mapping::volume_prune)
                .map_err(mapping::engine_error)
        })
    }
}
