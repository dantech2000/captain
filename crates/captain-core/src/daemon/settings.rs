use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use super::merge::daemon_json;

/// The TCP port that Docker uses by convention for the unencrypted API.
pub const DEFAULT_TCP_PORT: u16 = 2375;

/// What the user saved for Captain Engine's Docker daemon. The next start applies it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct DaemonSettings {
    /// Mirrors that Docker tries before Docker Hub, as `https://` or `http://`
    /// URLs. Applies at the next engine start.
    #[schemars(example = ["https://mirror.gcr.io"])]
    pub registry_mirrors: Vec<String>,
    /// Registries that Docker may reach over plain HTTP or with an untrusted
    /// certificate, as `host:port` or CIDR entries. Applies at the next engine start.
    #[schemars(example = ["registry.local:5000"])]
    pub insecure_registries: Vec<String>,
    /// Other `daemon.json` keys for the engine's Docker daemon. Docker checks them
    /// before it starts, and Captain rolls back keys it rejects. Applies at the next
    /// engine start.
    // See [`daemon_json`] for how they merge.
    #[schemars(example = serde_json::json!({"log-level": "warn"}))]
    pub custom: Map<String, Value>,
    /// Also serve the Docker API on `tcp://127.0.0.1:<tcp_port>`, without TLS.
    /// Applies at the next engine start.
    #[schemars(example = true)]
    pub tcp: bool,
    /// The port for `tcp`. Captain keeps it while `tcp` is off. Applies at the next
    /// engine start.
    #[schemars(range(min = 1), example = 23750)]
    pub tcp_port: u16,
}

impl Default for DaemonSettings {
    fn default() -> Self {
        Self {
            registry_mirrors: Vec::new(),
            insecure_registries: Vec::new(),
            custom: Map::new(),
            tcp: false,
            tcp_port: DEFAULT_TCP_PORT,
        }
    }
}

impl DaemonSettings {
    /// The TCP port the engine listens on, or `None` when the switch is off.
    pub fn tcp_port(&self) -> Option<u16> {
        self.tcp.then_some(self.tcp_port)
    }

    /// The custom keys as indented JSON for the form, or empty text when there are none.
    pub fn custom_text(&self) -> String {
        if self.custom.is_empty() {
            return String::new();
        }
        serde_json::to_string_pretty(&self.custom).unwrap_or_default()
    }

    /// The files these settings produce in the VM.
    pub fn state(&self) -> DaemonState {
        DaemonState {
            daemon_json: daemon_json(self),
            tcp_port: self.tcp_port(),
        }
    }
}

/// What a running engine uses: the contents of `/etc/docker/daemon.json` and the
/// TCP port. Two states are equal when their JSON values are, whatever the formatting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DaemonState {
    pub daemon_json: Value,
    pub tcp_port: Option<u16>,
}
