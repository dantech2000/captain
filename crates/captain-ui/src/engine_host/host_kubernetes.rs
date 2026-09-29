//! The Kubernetes side of [`HostModel`]. See ADR 0010.

use std::sync::Arc;

use captain_core::kubernetes::{KubernetesHost, KubernetesSettings};

use super::HostModel;

impl HostModel {
    /// The k3s cluster of Captain Engine, when the host has one.
    pub fn kubernetes(&self) -> Option<Arc<dyn KubernetesHost>> {
        self.host.kubernetes()
    }

    /// Hands new Kubernetes settings to the host for its next start.
    pub fn set_kubernetes(&self, settings: KubernetesSettings) {
        self.host.set_kubernetes(settings);
    }
}
