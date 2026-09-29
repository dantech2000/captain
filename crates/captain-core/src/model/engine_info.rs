/// Facts about the connected engine.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EngineInfo {
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
