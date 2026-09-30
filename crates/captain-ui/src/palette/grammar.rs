use captain_core::grammar::{Catalog, ParseError, Verb, complete, parse};
use captain_core::kubernetes::KubernetesStatus;
use captain_core::store::GroupKey;
use gpui_kit::*;

use super::ranking::Ranked;
use super::suggestion_row;
use crate::kubernetes::kubernetes_model;
use crate::port_forwarding::ForwardingModel;
use crate::theme::Palette;
use crate::workspace::{Page, Workspace};

/// The live names the grammar resolves: containers, Compose services and projects,
/// and, while k3s runs, the Kubernetes services in `forwarding`.
pub fn catalog(workspace: &Workspace, forwarding: Option<&ForwardingModel>, cx: &App) -> Catalog {
    let kubernetes = kube_runs(cx);
    let current_project = match (workspace.page(), workspace.focus()) {
        (Page::Project, Some(GroupKey::Project(project))) => Some(project.clone()),
        _ => workspace.project_filter().map(str::to_string),
    };
    Catalog {
        containers: workspace.store().containers().to_vec(),
        kube_services: forwarding
            .filter(|_| kubernetes)
            .map(|model| model.services().to_vec())
            .unwrap_or_default(),
        current_project,
        kubernetes,
        compose: workspace.has_project_runner(),
    }
}

/// True while k3s runs.
pub fn kube_runs(cx: &App) -> bool {
    kubernetes_model(cx)
        .is_some_and(|model| matches!(model.read(cx).status(), KubernetesStatus::Running { .. }))
}

/// True once the first word is a verb and more follows. The palette then lists only
/// the grammar's rows, not the plain search results.
pub fn is_command(query: &str) -> bool {
    let mut words = query.split_whitespace();
    words.next().and_then(Verb::parse).is_some()
        && (words.next().is_some() || query.ends_with(char::is_whitespace))
}

/// The grammar's rows for `query`, best first.
pub fn rows(
    query: &str,
    catalog: &Catalog,
    workspace: &Workspace,
    palette: &Palette,
) -> Vec<Ranked> {
    complete(query, catalog)
        .into_iter()
        .map(|suggestion| suggestion_row::ranked(suggestion, catalog, workspace, palette))
        .collect()
}

/// Why a command does not run, for the palette to show when no row fits.
pub fn error(query: &str, catalog: &Catalog) -> Option<String> {
    if !is_command(query) {
        return None;
    }
    match parse(query, catalog) {
        Err(ParseError::NotACommand) | Ok(_) => None,
        Err(error) => Some(error.message()),
    }
}
