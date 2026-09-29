//! [`Clients`]: the kube client of a port forward, rebuilt when the kubeconfig
//! changes. Reset Kubernetes writes new certificates, so a client from before it
//! fails TLS against the new cluster.

use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

use captain_core::HostError;
use kube::Client;

use crate::client::client;

pub struct Clients {
    kubeconfig: PathBuf,
    /// The kubeconfig text and the client built from it.
    current: Mutex<Option<(Vec<u8>, Client)>>,
}

impl Clients {
    pub fn new(kubeconfig: PathBuf) -> Self {
        Self {
            kubeconfig,
            current: Mutex::new(None),
        }
    }

    /// The client for the kubeconfig as it is now. The file is small, so each call
    /// reads it.
    pub async fn get(&self) -> Result<Client, HostError> {
        let text = std::fs::read(&self.kubeconfig).map_err(|error| {
            HostError(format!(
                "Cannot read {}: {error}. Is Kubernetes on?",
                self.kubeconfig.display()
            ))
        })?;
        if let Some((seen, client)) = lock(&self.current).as_ref()
            && *seen == text
        {
            return Ok(client.clone());
        }
        let fresh = client(&self.kubeconfig).await?;
        *lock(&self.current) = Some((text, fresh.clone()));
        Ok(fresh)
    }
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
mod tests;
