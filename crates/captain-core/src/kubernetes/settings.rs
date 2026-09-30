use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::GIB;

/// The port of the Kubernetes API on the Mac, as in k3s and Rancher Desktop.
pub const DEFAULT_PORT: u16 = 6443;

/// The engine memory Captain recommends with Kubernetes on. k3s needs 2 GB of its
/// own (https://docs.k3s.io/installation/requirements).
pub const RECOMMENDED_MEMORY: u64 = 6 * GIB;

/// What the user saved for Kubernetes. The next engine start, or Apply, uses it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct KubernetesSettings {
    /// Run a k3s cluster in Captain Engine. Give the engine 6 GiB of memory or
    /// more. Applies at the next engine start.
    #[schemars(example = true)]
    pub enabled: bool,
    /// The k3s version. When you turn Kubernetes on, Captain saves the stable
    /// version, so k3s never upgrades by itself. Applies at the next engine start.
    #[schemars(example = "v1.36.4+k3s1")]
    pub version: Option<String>,
    /// The port of the Kubernetes API on `127.0.0.1`. Applies at the next engine
    /// start.
    #[schemars(range(min = 1), example = 16443)]
    pub port: u16,
    /// Install Traefik, the k3s ingress controller, on ports 80 and 443. Applies at
    /// the next engine start.
    #[schemars(example = false)]
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
