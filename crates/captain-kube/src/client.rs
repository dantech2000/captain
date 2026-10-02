//! A kube client for the `captain-desktop` context in Captain's own kubeconfig.

use std::path::Path;

use captain_core::HostError;
use captain_core::kubernetes::CONTEXT;
use kube::config::{KubeConfigOptions, Kubeconfig};
use kube::{Client, Config};

pub async fn client(kubeconfig: &Path) -> Result<Client, HostError> {
    let file = Kubeconfig::read_from(kubeconfig).map_err(|error| {
        HostError(format!(
            "Cannot read {}: {error}. Is Kubernetes on?",
            kubeconfig.display()
        ))
    })?;
    // A file from before the rename has only the old `captain` context, which is its
    // current context, until the next start rewrites it.
    let named = file.contexts.iter().any(|context| context.name == CONTEXT);
    let options = KubeConfigOptions {
        context: named.then(|| CONTEXT.into()),
        ..KubeConfigOptions::default()
    };
    let config = Config::from_custom_kubeconfig(file, &options)
        .await
        .map_err(error)?;
    Client::try_from(config).map_err(error)
}

/// A kube error as a message for the user.
pub fn error(error: impl std::fmt::Display) -> HostError {
    HostError(format!("Kubernetes: {error}"))
}
