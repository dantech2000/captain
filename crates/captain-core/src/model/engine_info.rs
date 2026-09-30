/// Facts about the connected engine.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EngineInfo {
    /// The daemon's ID. It stays the same when the engine restarts. Empty if the
    /// engine does not report it.
    pub id: String,
    pub version: String,
    pub api_version: String,
    pub os: String,
    pub arch: String,
    /// Where Captain connected, for example `unix:///var/run/docker.sock`.
    pub endpoint: String,
    /// CPUs available to the engine.
    pub cpus: u32,
    /// Memory available to the engine, in bytes.
    pub memory_bytes: u64,
}

impl EngineInfo {
    /// True if `other` came from the same daemon at the same endpoint.
    pub fn same_daemon(&self, other: &EngineInfo) -> bool {
        self.id == other.id && self.endpoint == other.endpoint
    }
}
