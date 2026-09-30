use crate::model::EnvVar;

/// What the map needs to know about one container.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MapService {
    pub id: String,
    /// The Compose service, else the container name.
    pub name: String,
    /// The container name. Other services on the network reach it by this name too.
    pub container_name: String,
    /// The networks it is attached to. Empty until it is inspected.
    pub networks: Vec<String>,
    /// The published host ports.
    pub ports: Vec<u16>,
    /// The volumes it mounts, by name. Bind mounts are left out.
    pub volumes: Vec<String>,
    pub env: Vec<EnvVar>,
}
