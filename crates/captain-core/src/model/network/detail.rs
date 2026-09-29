use super::{Network, NetworkEndpoint};

/// One IPAM pool of a network.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Subnet {
    /// For example `172.18.0.0/16`.
    pub subnet: String,
    pub gateway: Option<String>,
}

/// A network with everything the network inspector shows.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NetworkDetail {
    /// The list fields. `containers` is the number of endpoints.
    pub network: Network,
    /// Creation time in RFC 3339 format, as the engine reports it. Empty if unknown.
    pub created: String,
    /// Every IPAM pool, IPv4 and IPv6, in engine order.
    pub subnets: Vec<Subnet>,
    /// True if standalone containers can attach to the network.
    pub attachable: bool,
    pub ipv6: bool,
    /// Attached containers, sorted by name.
    pub endpoints: Vec<NetworkEndpoint>,
}

impl NetworkDetail {
    /// A detail with only the list fields, for engines that do not report more.
    pub fn from_network(network: Network) -> Self {
        Self {
            subnets: network
                .subnet
                .clone()
                .map(|subnet| Subnet {
                    subnet,
                    gateway: network.gateway.clone(),
                })
                .into_iter()
                .collect(),
            network,
            ..Self::default()
        }
    }

    /// The subnets as one line, for example `172.18.0.0/16, fd00::/64`, or `—`.
    pub fn subnets_label(&self) -> String {
        let subnets: Vec<&str> = self.subnets.iter().map(|s| s.subnet.as_str()).collect();
        dash_if_empty(subnets)
    }

    /// The gateways as one line, or `—`.
    pub fn gateways_label(&self) -> String {
        let gateways: Vec<&str> = self
            .subnets
            .iter()
            .filter_map(|s| s.gateway.as_deref())
            .collect();
        dash_if_empty(gateways)
    }

    /// The date part of [`NetworkDetail::created`], for example `2026-09-28`, or `—`.
    pub fn created_date(&self) -> &str {
        match self.created.get(..10) {
            Some(date) => date,
            None if self.created.is_empty() => "—",
            None => &self.created,
        }
    }
}

fn dash_if_empty(items: Vec<&str>) -> String {
    if items.is_empty() {
        "—".into()
    } else {
        items.join(", ")
    }
}
