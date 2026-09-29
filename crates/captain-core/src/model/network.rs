//! Network models: the list entry and the inspect detail with attached containers.

mod detail;
mod endpoint;

use std::collections::BTreeMap;

pub use detail::{NetworkDetail, Subnet};
pub use endpoint::NetworkEndpoint;

/// Networks that every Docker engine creates. They cannot be removed.
const BUILT_IN: [&str; 3] = ["bridge", "host", "none"];

/// A network as Captain shows it in lists.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Network {
    pub id: String,
    pub name: String,
    /// The network driver, for example `bridge` or `overlay`.
    pub driver: String,
    /// `local`, `global`, or `swarm`.
    pub scope: String,
    /// The first IPAM subnet, preferring IPv4, for example `172.18.0.0/16`.
    pub subnet: Option<String>,
    pub gateway: Option<String>,
    /// True if the network has no route to the outside.
    pub internal: bool,
    pub labels: BTreeMap<String, String>,
    /// The number of containers attached to the network.
    pub containers: usize,
    /// The Compose project, from the `com.docker.compose.project` label.
    pub compose_project: Option<String>,
}

impl Network {
    /// The first 12 characters of the ID, as the Docker CLI shows it.
    pub fn short_id(&self) -> &str {
        &self.id[..self.id.len().min(12)]
    }

    /// True for `bridge`, `host`, and `none`, which the engine creates itself.
    pub fn is_built_in(&self) -> bool {
        BUILT_IN.contains(&self.name.as_str())
    }

    /// True if at least one container is attached.
    pub fn is_in_use(&self) -> bool {
        self.containers > 0
    }

    /// Only networks that Captain's user made and that have no containers.
    pub fn can_remove(&self) -> bool {
        !self.is_built_in() && !self.is_in_use()
    }

    /// For example `3 containers` or `No containers`.
    pub fn usage_label(&self) -> String {
        match self.containers {
            0 => "No containers".into(),
            1 => "1 container".into(),
            n => format!("{n} containers"),
        }
    }

    /// Why Remove is off, or `None` if the network can be removed.
    pub fn remove_blocker(&self) -> Option<&'static str> {
        if self.is_built_in() {
            Some("Built-in networks cannot be removed")
        } else if self.is_in_use() {
            Some("Disconnect its containers first")
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests;
