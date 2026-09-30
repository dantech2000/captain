//! The Kubernetes group: its status line, a check item that turns it on or off, and
//! the Kubernetes Contexts submenu. See feature 0024.

use captain_core::kubernetes::KubernetesStatus;

use super::contexts::contexts_item;
use super::menu_model::{TrayCommand, TrayItem};
use super::snapshot::TraySnapshot;

/// Empty when Captain Engine has no cluster and the kubeconfig has no contexts.
pub fn kubernetes_items(snapshot: &TraySnapshot) -> Vec<TrayItem> {
    let mut items = Vec::new();
    if let Some(kube) = &snapshot.kubernetes {
        // muda's check items take no image, so the status dot has its own line.
        if kube.enabled || kube.status != KubernetesStatus::Off {
            items.push(TrayItem::Status {
                label: kube.line(),
                light: kube.light(),
            });
        }
        items.push(TrayItem::Check {
            label: "Kubernetes".into(),
            command: TrayCommand::SetKubernetes(!kube.enabled),
            checked: kube.enabled,
        });
    }
    items.extend(contexts_item(&snapshot.contexts));
    items
}

#[cfg(test)]
mod tests;
