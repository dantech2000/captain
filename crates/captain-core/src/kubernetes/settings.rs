use serde::{Deserialize, Serialize};

use crate::GIB;

/// The port of the Kubernetes API on the Mac, as in k3s and Rancher Desktop.
pub const DEFAULT_PORT: u16 = 6443;

/// The engine memory Captain recommends with Kubernetes on. k3s needs 2 GB of its
/// own (https://docs.k3s.io/installation/requirements).
pub const RECOMMENDED_MEMORY: u64 = 6 * GIB;

/// What the user saved for Kubernetes. The next engine start, or Apply, uses it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct KubernetesSettings {
    /// Run k3s in Captain Engine. Off by default.
    pub enabled: bool,
    /// The k3s version, for example `v1.36.4+k3s1`. `None` until the user turns
    /// Kubernetes on; then Captain saves the stable version, so it never upgrades by
    /// itself.
    pub version: Option<String>,
    /// The port of the Kubernetes API on `127.0.0.1`.
    pub port: u16,
    /// Install Traefik, k3s's ingress controller, on ports 80 and 443.
    pub traefik: bool,
}

impl Default for KubernetesSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            version: None,
            port: DEFAULT_PORT,
            traefik: true,
        }
    }
}
