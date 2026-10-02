//! [`KubeForwarder`]: the [`PortForwarding`] of Captain's `captain-desktop` context. Forwards
//! live on its own tokio runtime until they stop or Captain quits.

use std::collections::BTreeMap;
use std::future::Future;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};

use captain_core::kubernetes::{
    Forward, ForwardKey, KubeService, PortForwarding, check_local_port,
};
use captain_core::{HostError, HostFuture};
use futures::FutureExt;
use tokio::runtime::{Builder, Handle, Runtime};
use tokio::task::AbortHandle;

use crate::client::client;
use crate::clients::Clients;
use crate::{forward, services};

struct Running {
    local_port: u16,
    task: AbortHandle,
}

type Table = Arc<Mutex<BTreeMap<ForwardKey, Running>>>;

/// Services and port forwards for the cluster in `kubeconfig`.
pub struct KubeForwarder {
    runtime: Runtime,
    kubeconfig: PathBuf,
    clients: Arc<Clients>,
    forwards: Table,
}

impl KubeForwarder {
    /// A forwarder for the `captain-desktop` context in `kubeconfig`, which may not exist yet.
    pub fn new(kubeconfig: PathBuf) -> std::io::Result<Self> {
        let runtime = Builder::new_multi_thread()
            .worker_threads(1)
            .thread_name("captain-kube")
            .enable_all()
            .build()?;
        Ok(Self {
            runtime,
            clients: Arc::new(Clients::new(kubeconfig.clone())),
            kubeconfig,
            forwards: Table::default(),
        })
    }

    fn spawn<T, F>(&self, future: F) -> HostFuture<T>
    where
        T: Send + 'static,
        F: Future<Output = Result<T, HostError>> + Send + 'static,
    {
        spawn(self.runtime.handle(), future)
    }
}

impl PortForwarding for KubeForwarder {
    fn services(&self) -> HostFuture<Vec<KubeService>> {
        let path = self.kubeconfig.clone();
        self.spawn(async move { services::list(client(&path).await?).await })
    }

    fn forward(&self, key: ForwardKey, local_port: Option<u16>) -> HostFuture<Forward> {
        if let Some(port) = local_port
            && let Err(why) = check_local_port(port)
        {
            return futures::future::ready(Err(HostError(why))).boxed();
        }
        let (path, table, handle, clients) = (
            self.kubeconfig.clone(),
            self.forwards.clone(),
            self.runtime.handle().clone(),
            self.clients.clone(),
        );
        self.spawn(async move {
            if let Some(running) = lock(&table).get(&key) {
                return Ok(Forward {
                    local_port: running.local_port,
                    key,
                });
            }
            // Fails early when Kubernetes is off.
            client(&path).await?;
            let listener = forward::bind(local_port).await?;
            let local_port = listener
                .local_addr()
                .map_err(|error| HostError(error.to_string()))?
                .port();
            let task = handle
                .spawn(forward::serve(listener, clients, key.clone()))
                .abort_handle();
            Ok(claim(&table, key, Running { local_port, task }))
        })
    }

    fn stop(&self, key: &ForwardKey) {
        if let Some(running) = lock(&self.forwards).remove(key) {
            running.task.abort();
        }
    }

    fn forwards(&self) -> Vec<Forward> {
        lock(&self.forwards)
            .iter()
            .map(|(key, running)| Forward {
                key: key.clone(),
                local_port: running.local_port,
            })
            .collect()
    }
}

/// Records `running` under `key`. When a concurrent request for the same key got
/// there first, stops `running` and returns the recorded forward instead.
fn claim(table: &Table, key: ForwardKey, running: Running) -> Forward {
    let mut forwards = lock(table);
    if let Some(first) = forwards.get(&key) {
        running.task.abort();
        return Forward {
            local_port: first.local_port,
            key,
        };
    }
    let local_port = running.local_port;
    forwards.insert(key.clone(), running);
    Forward { key, local_port }
}

/// Runs `future` on tokio. Any executor can await the returned future.
fn spawn<T, F>(handle: &Handle, future: F) -> HostFuture<T>
where
    T: Send + 'static,
    F: Future<Output = Result<T, HostError>> + Send + 'static,
{
    handle
        .spawn(future)
        .map(|joined| joined.unwrap_or_else(|error| Err(HostError(error.to_string()))))
        .boxed()
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
mod tests;
