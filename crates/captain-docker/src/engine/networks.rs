//! Networks. Every call runs on the private tokio runtime.

use bollard::models::NetworkCreateRequest;
use bollard::query_parameters::PruneNetworksOptionsBuilder;
use captain_core::model::{Network, NetworkDetail};
use captain_core::{EngineFuture, NetworkApi};
use futures::future::join_all;

use super::DockerEngine;
use crate::{mapping, runtime};

impl NetworkApi for DockerEngine {
    fn list_networks(&self) -> EngineFuture<Vec<Network>> {
        let docker = self.docker.clone();
        runtime::spawn(self.runtime.handle(), async move {
            let networks = docker
                .list_networks(None)
                .await
                .map_err(mapping::engine_error)?;
            // The list leaves out attached containers, so inspect each network for them.
            // A network removed in between counts as having none.
            let counts = join_all(networks.iter().map(|network| {
                let id = network.id.clone().unwrap_or_default();
                let docker = docker.clone();
                async move {
                    docker
                        .inspect_network(&id, None)
                        .await
                        .ok()
                        .and_then(|inspect| inspect.containers)
                        .map_or(0, |containers| containers.len())
                }
            }))
            .await;
            Ok(networks
                .into_iter()
                .zip(counts)
                .map(|(network, count)| mapping::network(network, count))
                .collect())
        })
    }

    fn inspect_network(&self, id: &str) -> EngineFuture<NetworkDetail> {
        let docker = self.docker.clone();
        let id = id.to_string();
        runtime::spawn(self.runtime.handle(), async move {
            docker
                .inspect_network(&id, None)
                .await
                .map(mapping::network_detail)
                .map_err(mapping::engine_error)
        })
    }

    fn create_network(&self, name: &str) -> EngineFuture<String> {
        let docker = self.docker.clone();
        let request = NetworkCreateRequest {
            name: name.to_string(),
            driver: Some("bridge".into()),
            ..NetworkCreateRequest::default()
        };
        runtime::spawn(self.runtime.handle(), async move {
            docker
                .create_network(request)
                .await
                .map(|response| response.id)
                .map_err(mapping::engine_error)
        })
    }

    fn remove_network(&self, id: &str) -> EngineFuture<()> {
        let docker = self.docker.clone();
        let id = id.to_string();
        runtime::spawn(self.runtime.handle(), async move {
            docker
                .remove_network(&id)
                .await
                .map_err(mapping::engine_error)
        })
    }

    fn prune_unused_networks(&self, label: Option<&str>) -> EngineFuture<Vec<String>> {
        let docker = self.docker.clone();
        let filters = mapping::network_prune_filters(label);
        runtime::spawn(self.runtime.handle(), async move {
            let options = PruneNetworksOptionsBuilder::default()
                .filters(&filters)
                .build();
            docker
                .prune_networks(Some(options))
                .await
                .map(|response| response.networks_deleted.unwrap_or_default())
                .map_err(mapping::engine_error)
        })
    }
}
