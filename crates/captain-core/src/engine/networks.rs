use super::EngineFuture;
use crate::model::{Network, NetworkDetail};

/// Networks: list, inspect, create, remove, and prune.
pub trait NetworkApi {
    /// All networks, with the number of attached containers.
    fn list_networks(&self) -> EngineFuture<Vec<Network>>;

    /// One network with all its subnets and attached containers.
    fn inspect_network(&self, id: &str) -> EngineFuture<NetworkDetail>;

    /// Creates a network with the `bridge` driver and returns its ID.
    fn create_network(&self, name: &str) -> EngineFuture<String>;

    /// Removes a network. The engine refuses if containers are attached.
    fn remove_network(&self, id: &str) -> EngineFuture<()>;

    /// Removes custom networks that have no containers, and returns their names.
    /// Built-in networks stay. `label` limits the prune to networks with that label,
    /// as `key` or `key=value`.
    fn prune_unused_networks(&self, label: Option<&str>) -> EngineFuture<Vec<String>>;
}
