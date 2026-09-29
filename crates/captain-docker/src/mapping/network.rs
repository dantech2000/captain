//! Converts bollard network types.

mod detail;
mod prune;

use std::collections::BTreeMap;

use bollard::models::{Ipam, Network as DockerNetwork};
use captain_core::model::Network;

pub use detail::network_detail;
pub use prune::network_prune_filters;

const COMPOSE_PROJECT_LABEL: &str = "com.docker.compose.project";

/// A network from the list, with the container count from its inspect response.
pub fn network(network: DockerNetwork, containers: usize) -> Network {
    let (subnet, gateway) = network.ipam.as_ref().map(subnet).unwrap_or_default();
    let labels: BTreeMap<_, _> = network.labels.unwrap_or_default().into_iter().collect();
    Network {
        id: network.id.unwrap_or_default(),
        name: network.name.unwrap_or_default(),
        driver: network.driver.unwrap_or_default(),
        scope: network.scope.unwrap_or_default(),
        subnet,
        gateway,
        internal: network.internal.unwrap_or_default(),
        containers,
        compose_project: labels.get(COMPOSE_PROJECT_LABEL).cloned(),
        labels,
    }
}

/// The first subnet and its gateway. IPv4 wins over IPv6.
fn subnet(ipam: &Ipam) -> (Option<String>, Option<String>) {
    let configs = ipam.config.as_deref().unwrap_or_default();
    let with_subnet = || configs.iter().filter(|c| c.subnet.is_some());
    let config = with_subnet()
        .find(|c| c.subnet.as_deref().is_some_and(|s| !s.contains(':')))
        .or_else(|| with_subnet().next());
    match config {
        Some(config) => (
            config.subnet.clone(),
            config.gateway.clone().filter(|g| !g.is_empty()),
        ),
        None => (None, None),
    }
}

#[cfg(test)]
mod tests;
