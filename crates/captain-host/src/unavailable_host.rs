//! A placeholder where Captain Engine cannot run yet, such as Windows (a WSL2 host
//! comes later, ADR 0008).

use captain_core::{EngineHost, HostError, HostFuture, HostResources, HostStatus, HostStream};
use futures::{FutureExt, StreamExt, future, stream};

/// Always `NotInstalled`; every action fails with the same reason.
pub struct UnavailableHost {
    reason: String,
    resources: HostResources,
}

impl UnavailableHost {
    pub fn new(reason: impl Into<String>, resources: HostResources) -> Self {
        Self {
            reason: reason.into(),
            resources,
        }
    }

    fn fail<T: Send + 'static>(&self) -> HostFuture<T> {
        future::ready(Err(HostError(self.reason.clone()))).boxed()
    }
}

impl EngineHost for UnavailableHost {
    fn can_control(&self) -> bool {
        false
    }

    fn supported(&self) -> bool {
        false
    }

    fn status(&self) -> HostFuture<HostStatus> {
        future::ready(Ok(HostStatus::NotInstalled(self.reason.clone()))).boxed()
    }

    fn start(&self) -> HostStream<String> {
        stream::once(future::ready(Err(HostError(self.reason.clone())))).boxed()
    }

    fn stop(&self) -> HostFuture<()> {
        self.fail()
    }

    fn endpoint(&self) -> Option<String> {
        None
    }

    fn resources(&self) -> HostResources {
        self.resources
    }

    fn set_resources(&self, _: HostResources) -> HostFuture<()> {
        self.fail()
    }

    fn reset(&self) -> HostFuture<()> {
        self.fail()
    }
}
