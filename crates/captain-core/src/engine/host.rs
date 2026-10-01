//! The machine that runs the engine: on macOS a Lima VM that Captain starts and
//! stops. See docs/adr/0008-captain-engine.md.

mod resources;
mod status;

use futures::future::BoxFuture;
use futures::stream::BoxStream;

pub use resources::{GIB, HostResources};
pub use status::HostStatus;

use std::path::PathBuf;
use std::sync::Arc;

use crate::daemon::{DaemonSettings, DaemonState};
use crate::kubernetes::{KubernetesHost, KubernetesSettings};
use crate::snapshot::EngineSnapshots;

/// Why a host action failed. The message is for the user.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{0}")]
pub struct HostError(pub String);

/// A future that any executor can await.
pub type HostFuture<T> = BoxFuture<'static, Result<T, HostError>>;
/// A stream that any executor can poll.
pub type HostStream<T> = BoxStream<'static, Result<T, HostError>>;

/// Starts, stops, and configures the engine's machine. The UI talks only to this
/// trait, so a different VM can replace Lima later. Like [`crate::Engine`], the
/// futures and streams must not depend on a specific async runtime.
pub trait EngineHost: Send + Sync + 'static {
    /// False for a host that only reports an engine it does not control, such as
    /// the system `dockerd` on Linux.
    fn can_control(&self) -> bool;

    /// The current state. It may run a command, so it can take a moment.
    fn status(&self) -> HostFuture<HostStatus>;

    /// Creates the machine if needed and starts it. The stream yields progress lines.
    /// It ends after the engine runs, or with one error.
    fn start(&self) -> HostStream<String>;

    /// Stops the machine. The future ends when it has stopped.
    fn stop(&self) -> HostFuture<()>;

    /// The `DOCKER_HOST`-style URL of the engine, valid while it runs.
    fn endpoint(&self) -> Option<String>;

    /// The resources that the next start uses.
    fn resources(&self) -> HostResources;

    /// Changes the resources. A running machine gets them on its next start.
    fn set_resources(&self, resources: HostResources) -> HostFuture<()>;

    /// Deletes the machine with all its containers, images, and volumes.
    fn reset(&self) -> HostFuture<()>;

    /// Changes the Docker daemon settings. The next start applies them.
    fn set_daemon(&self, _daemon: DaemonSettings) {}

    /// The daemon settings the running engine uses, when the host knows them. The
    /// UI compares them with the saved ones to ask for a restart.
    fn running_daemon(&self) -> Option<DaemonState> {
        None
    }

    /// The resources the running machine started with, as of the last status check.
    /// `None` while it does not run or the host does not know. The UI compares them
    /// with the saved ones to ask for a restart.
    fn running_resources(&self) -> Option<HostResources> {
        None
    }

    /// The size of the machine's disk now, as of the last status check, or `None`
    /// before the first setup. A disk cannot shrink below it.
    fn current_disk(&self) -> Option<u64> {
        None
    }

    /// Snapshots of this machine, or `None` for a host without them. See ADR 0012.
    fn snapshots(&self) -> Option<Arc<dyn EngineSnapshots>> {
        None
    }

    /// Changes the Kubernetes settings. The next start applies them. See ADR 0010.
    fn set_kubernetes(&self, _kubernetes: KubernetesSettings) {}

    /// The k3s cluster in this machine, or `None` for a host without one.
    fn kubernetes(&self) -> Option<Arc<dyn KubernetesHost>> {
        None
    }

    /// The folder with the machine's files, for Show engine files, or `None`.
    fn files_dir(&self) -> Option<PathBuf> {
        None
    }

    /// What runs the machine and its version, such as `Lima 2.0.3`, for the About
    /// line. It reads a file, so it is quick.
    fn runtime_version(&self) -> Option<String> {
        None
    }

    /// Free bytes on the disk that holds the machine. It may run a command, so the
    /// UI calls it in the background.
    fn free_disk(&self) -> Option<u64> {
        None
    }
}
