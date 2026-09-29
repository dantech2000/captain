//! The Kubernetes Contexts submenu: every context in the user's kubeconfig, with a
//! check mark on the current one. Picking one makes it current, like
//! `kubectl config use-context`. See feature 0024.

use captain_core::kubernetes::KubeContexts;

use super::menu_model::{TrayCommand, TrayItem};

/// The submenu, or `None` when the kubeconfig has no contexts.
pub fn contexts_item(contexts: &KubeContexts) -> Option<TrayItem> {
    if contexts.names.is_empty() {
        return None;
    }
    let items = contexts
        .names
        .iter()
        .map(|name| TrayItem::Check {
            label: name.clone(),
            command: TrayCommand::UseContext(name.clone()),
            checked: contexts.current.as_ref() == Some(name),
        })
        .collect();
    Some(TrayItem::Submenu {
        label: "Kubernetes Contexts".into(),
        items,
    })
}

#[cfg(test)]
mod tests;
