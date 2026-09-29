/// Facts about the connected engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineInfo {
    pub version: String,
    pub api_version: String,
    pub os: String,
    pub arch: String,
    /// Where Captain connected, for example `unix:///var/run/docker.sock`.
    pub endpoint: String,
}
