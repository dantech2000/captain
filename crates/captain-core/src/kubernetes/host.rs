use std::path::PathBuf;

use super::{KubernetesSettings, VersionList};
use crate::{HostFuture, HostStream};

/// The state of the k3s cluster in Captain Engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KubernetesStatus {
    /// Kubernetes is off, or the engine does not run.
    Off,
    Starting,
    /// k3s runs `version`.
    Running {
        version: String,
    },
    /// k3s did not start. The text says why.
    Failed(String),
}

impl KubernetesStatus {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::Starting => "Starting",
            Self::Running { .. } => "Running",
            Self::Failed(_) => "Failed",
        }
    }

    pub fn is_running(&self) -> bool {
        matches!(self, Self::Running { .. })
    }

    /// The version k3s runs, while it runs.
    pub fn version(&self) -> Option<&str> {
        match self {
            Self::Running { version } => Some(version),
            _ => None,
        }
    }
}

/// Runs k3s in the engine's machine. Like [`crate::EngineHost`], the futures and
/// streams must not depend on a specific async runtime. See ADR 0010.
pub trait KubernetesHost: Send + Sync + 'static {
    /// The cluster's state. It asks the machine, so it can take a moment.
    fn status(&self) -> HostFuture<KubernetesStatus>;

    /// The versions to offer. `refresh` asks the network; otherwise the cache
    /// answers when it can. Downloaded versions are always in the list.
    fn versions(&self, refresh: bool) -> HostFuture<VersionList>;

    /// Downloads `settings.version` if needed, installs it in the running engine,
    /// and starts k3s. The stream yields progress lines and ends once the API
    /// answers, or with one error. A lower version than the installed one fails;
    /// [`KubernetesHost::reset`] downgrades.
    fn enable(&self, settings: KubernetesSettings) -> HostStream<String>;

    /// Stops k3s and keeps its state for the next enable.
    fn disable(&self) -> HostFuture<()>;

    /// Stops k3s, deletes the cluster state and the pod containers, and enables it
    /// again with the saved settings when they turn it on. Images stay.
    fn reset(&self) -> HostStream<String>;

    /// Captain's own kubeconfig, `~/.captain/kubeconfig`, with only the `captain`
    /// context.
    fn kubeconfig(&self) -> PathBuf;
}
