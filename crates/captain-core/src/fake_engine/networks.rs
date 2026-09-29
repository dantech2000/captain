use std::collections::BTreeMap;

use futures::FutureExt;
use futures::future::ready;

use super::FakeEngine;
use crate::model::{Network, NetworkDetail, NetworkEndpoint};
use crate::store::{prunable_networks, validate_name};
use crate::{EngineError, EngineFuture, NetworkApi};

/// The fake engine's networks data. Create, remove, and prune check their input the
/// way the engine does, but do not change the list.
#[derive(Debug, Clone, Default)]
pub struct FakeNetworks {
    pub networks: Vec<Network>,
    /// The attached containers of each network, by network ID.
    pub endpoints: BTreeMap<String, Vec<NetworkEndpoint>>,
}

impl NetworkApi for FakeEngine {
    fn list_networks(&self) -> EngineFuture<Vec<Network>> {
        ready(Ok(self.networks.networks.clone())).boxed()
    }

    fn inspect_network(&self, id: &str) -> EngineFuture<NetworkDetail> {
        let result = match self.networks.networks.iter().find(|n| n.id == id) {
            None => Err(EngineError::Api(format!("no such network: {id}"))),
            Some(network) => Ok(NetworkDetail {
                endpoints: self.networks.endpoints.get(id).cloned().unwrap_or_default(),
                ..NetworkDetail::from_network(network.clone())
            }),
        };
        ready(result).boxed()
    }

    fn create_network(&self, name: &str) -> EngineFuture<String> {
        let result = match validate_name(name) {
            Err(error) => Err(EngineError::Api(error.to_string())),
            Ok(()) if self.networks.networks.iter().any(|n| n.name == name) => {
                Err(EngineError::Api(format!("network {name} already exists")))
            }
            Ok(()) => Ok(format!("fake-{name}")),
        };
        ready(result).boxed()
    }

    fn remove_network(&self, id: &str) -> EngineFuture<()> {
        let result = match self.networks.networks.iter().find(|n| n.id == id) {
            None => Err(EngineError::Api(format!("no such network: {id}"))),
            Some(network) if network.is_built_in() => Err(EngineError::Api(format!(
                "{} is a pre-defined network and cannot be removed",
                network.name
            ))),
            Some(network) if network.is_in_use() => Err(EngineError::Api(format!(
                "network {} has active endpoints",
                network.name
            ))),
            Some(_) => Ok(()),
        };
        ready(result).boxed()
    }

    fn prune_unused_networks(&self, label: Option<&str>) -> EngineFuture<Vec<String>> {
        let removed = prunable_networks(&self.networks.networks, label)
            .into_iter()
            .map(|n| n.name.clone())
            .collect();
        ready(Ok(removed)).boxed()
    }
}
