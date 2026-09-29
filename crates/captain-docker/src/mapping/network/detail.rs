use bollard::models::{EndpointResource, Network as DockerNetwork, NetworkInspect};
use captain_core::model::{NetworkDetail, NetworkEndpoint, Subnet};

use super::network;

/// A network inspect response, with every subnet and the attached containers.
pub fn network_detail(inspect: NetworkInspect) -> NetworkDetail {
    let subnets = inspect
        .ipam
        .as_ref()
        .and_then(|ipam| ipam.config.as_ref())
        .into_iter()
        .flatten()
        .filter_map(|config| {
            Some(Subnet {
                subnet: config.subnet.clone()?,
                gateway: config.gateway.clone().filter(|g| !g.is_empty()),
            })
        })
        .collect();
    let mut endpoints: Vec<NetworkEndpoint> = inspect
        .containers
        .unwrap_or_default()
        .into_iter()
        .map(|(id, resource)| endpoint(id, resource))
        .collect();
    endpoints.sort_by(|a, b| a.name.cmp(&b.name));
    let list = DockerNetwork {
        id: inspect.id,
        name: inspect.name,
        scope: inspect.scope,
        driver: inspect.driver,
        ipam: inspect.ipam,
        internal: inspect.internal,
        labels: inspect.labels,
        ..DockerNetwork::default()
    };

    NetworkDetail {
        network: network(list, endpoints.len()),
        created: inspect.created.unwrap_or_default(),
        subnets,
        attachable: inspect.attachable.unwrap_or_default(),
        ipv6: inspect.enable_ipv6.unwrap_or_default(),
        endpoints,
    }
}

fn endpoint(container_id: String, resource: EndpointResource) -> NetworkEndpoint {
    // The engine sends an empty string for a missing address.
    let present = |value: Option<String>| value.filter(|v| !v.is_empty());
    NetworkEndpoint {
        container_id,
        name: resource.name.unwrap_or_default(),
        ipv4: present(resource.ipv4_address),
        ipv6: present(resource.ipv6_address),
        mac: present(resource.mac_address),
    }
}
