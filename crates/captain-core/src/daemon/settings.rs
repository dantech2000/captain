use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use super::merge::daemon_json;

/// The TCP port that Docker uses by convention for the unencrypted API.
pub const DEFAULT_TCP_PORT: u16 = 2375;

/// What the user saved for Captain Engine's Docker daemon. The next start applies it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct DaemonSettings {
    /// `registry-mirrors`: `https://` or `http://` URLs.
    pub registry_mirrors: Vec<String>,
    /// `insecure-registries`: `host:port` or CIDR entries.
    pub insecure_registries: Vec<String>,
    /// Other `daemon.json` keys. See [`daemon_json`] for how they merge.
    pub custom: Map<String, Value>,
    /// Listen on `tcp://127.0.0.1:<tcp_port>` on the Mac, without TLS.
    pub tcp: bool,
    /// Kept while `tcp` is off, so the switch remembers it.
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
