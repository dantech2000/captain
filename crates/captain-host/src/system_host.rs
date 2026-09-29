//! Linux needs no VM: [`SystemHost`] reports the system `dockerd`, which the
//! system starts and stops. See ADR 0008.

use std::path::{Path, PathBuf};

use captain_core::{EngineHost, HostError, HostFuture, HostResources, HostStatus, HostStream};
use futures::{FutureExt, StreamExt, future, stream};

const SYSTEM_SOCKET: &str = "/var/run/docker.sock";

/// The system Docker engine. Captain connects to it but does not control it.
pub struct SystemHost {
    socket: PathBuf,
    resources: HostResources,
}

impl SystemHost {
    pub fn new(resources: HostResources) -> Self {
        Self {
            socket: PathBuf::from(SYSTEM_SOCKET),
            resources,
        }
    }

    fn refuse<T: Send + 'static>(&self, action: &str) -> HostFuture<T> {
        let message = format!("Captain cannot {action} the system engine. Use systemctl.");
        future::ready(Err(HostError(message))).boxed()
    }
}

/// `Running` when the socket exists, else `Stopped`.
pub fn socket_status(socket: &Path) -> HostStatus {
    if socket.exists() {
        HostStatus::Running
    } else {
        HostStatus::Stopped
    }
}

impl EngineHost for SystemHost {
    fn can_control(&self) -> bool {
        false
    }

    fn status(&self) -> HostFuture<HostStatus> {
        future::ready(Ok(socket_status(&self.socket))).boxed()
    }

    fn start(&self) -> HostStream<String> {
        let message = "Captain cannot start the system engine. Run `sudo systemctl start docker`.";
        stream::once(future::ready(Err(HostError(message.into())))).boxed()
    }

    fn stop(&self) -> HostFuture<()> {
        self.refuse("stop")
    }

    fn endpoint(&self) -> Option<String> {
        Some(format!("unix://{}", self.socket.display()))
    }

    fn resources(&self) -> HostResources {
        self.resources
    }

    fn set_resources(&self, _: HostResources) -> HostFuture<()> {
        self.refuse("resize")
    }

    fn reset(&self) -> HostFuture<()> {
        self.refuse("reset")
    }
}

#[cfg(test)]
mod tests;
