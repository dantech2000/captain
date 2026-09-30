//! Gathers the menu's snapshot from the workspace, Captain Engine, the diagnostics
//! checks, and Kubernetes.

use std::collections::HashMap;

use captain_core::kubernetes::KubeContexts;
use captain_core::problems::ExitFacts;
use captain_ui::Workspace;
use gpui_kit::*;

use super::entries::KubeEntry;
use super::snapshot::TraySnapshot;

/// `facts` is what `inspect` said about each crashing container.
pub fn snapshot(
    workspace: &Entity<Workspace>,
    contexts: &KubeContexts,
    facts: &HashMap<String, ExitFacts>,
    cx: &App,
) -> TraySnapshot {
    let host = captain_ui::host_summary(cx);
    let checks = captain_ui::diagnostics_model(cx)
        .map(|model| model.read(cx).checks().to_vec())
        .unwrap_or_default();
    // Kubernetes shows where Settings shows its switch: Captain controls the engine.
    let kubernetes = host
        .as_ref()
        .filter(|host| host.can_control)
        .and(captain_ui::kubernetes_model(cx))
        .map(|model| {
            let model = model.read(cx);
            KubeEntry {
                enabled: model.settings(cx).enabled,
                status: model.status().clone(),
            }
        });
    TraySnapshot {
        contexts: contexts.clone(),
        kubernetes,
        ..TraySnapshot::of(workspace.read(cx), host.as_ref(), &checks, facts)
    }
}
