//! The `list_projects` answer: one row per Compose project, Kubernetes namespace,
//! and the loose containers, as the sidebar groups them.

use std::collections::{BTreeMap, HashMap};

use schemars::JsonSchema;
use serde::Serialize;

use super::container_rows::port_links;
use crate::model::{Container, Health};
use crate::problems::{ExitFacts, first_problem};
use crate::store::{Crash, GroupKey};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct ProjectList {
    pub projects: Vec<ProjectRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct ProjectRow {
    /// The Compose project, `k8s:<namespace>`, or `Loose containers`.
    pub name: String,
    /// compose, kubernetes, or loose.
    pub kind: String,
    /// Containers that run, are paused, or restart.
    pub running: usize,
    pub total: usize,
    /// The Compose services, or the containers of the other kinds.
    pub services: Vec<String>,
    /// For example `2 healthy, 1 unhealthy`; absent without health checks.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health: Option<String>,
    /// Published ports: a URL for web ports, else `localhost:PORT (Service)`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub urls: Vec<String>,
    /// The worst problem, in the words of Captain's menu bar.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub problem: Option<String>,
    /// The folder `docker compose` ran in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub folder: Option<String>,
}

/// The rows of `containers`, Compose projects first, then namespaces, then the
/// loose containers. `facts` and `crash` feed the problem line, as in the menu.
pub fn project_list(
    containers: &[Container],
    facts: &HashMap<String, ExitFacts>,
    crash: &dyn Fn(&str) -> Option<Crash>,
) -> ProjectList {
    let mut groups: BTreeMap<GroupKey, Vec<Container>> = BTreeMap::new();
    for container in containers.iter().filter(|c| !c.is_sandbox()) {
        groups
            .entry(GroupKey::of(container))
            .or_default()
            .push(container.clone());
    }
    let projects = groups
        .into_iter()
        .map(|(key, members)| row(key, &members, facts, crash))
        .collect();
    ProjectList { projects }
}

fn row(
    key: GroupKey,
    members: &[Container],
    facts: &HashMap<String, ExitFacts>,
    crash: &dyn Fn(&str) -> Option<Crash>,
) -> ProjectRow {
    let (name, kind) = match key {
        GroupKey::Project(name) => (name, "compose"),
        GroupKey::Namespace(namespace) => (format!("k8s:{namespace}"), "kubernetes"),
        GroupKey::Standalone => ("Loose containers".into(), "loose"),
    };
    let mut services: Vec<String> = members
        .iter()
        .map(|c| match (kind, &c.compose.service) {
            ("compose", Some(service)) => service.clone(),
            _ => c.display_name(),
        })
        .collect();
    services.sort();
    services.dedup();
    let mut urls: Vec<String> = Vec::new();
    for link in members.iter().flat_map(port_links) {
        if !urls.contains(&link) {
            urls.push(link);
        }
    }
    ProjectRow {
        name,
        kind: kind.into(),
        running: members.iter().filter(|c| c.state.is_active()).count(),
        total: members.len(),
        services,
        health: health(members),
        urls,
        problem: first_problem(None, &[], members, facts, crash).map(|p| p.line()),
        folder: members.iter().find_map(|c| c.compose.working_dir.clone()),
    }
}

/// `2 healthy, 1 unhealthy`, or `None` when no container has a health check.
fn health(members: &[Container]) -> Option<String> {
    let parts: Vec<String> = [Health::Healthy, Health::Starting, Health::Unhealthy]
        .into_iter()
        .filter_map(|health| {
            let count = members.iter().filter(|c| c.health == Some(health)).count();
            (count > 0).then(|| format!("{count} {}", health.label()))
        })
        .collect();
    (!parts.is_empty()).then(|| parts.join(", "))
}

impl ProjectList {
    /// One line per row, for clients that show only text.
    pub fn text(&self) -> String {
        let mut text = format!("{} projects.", self.projects.len());
        for row in &self.projects {
            let mut parts = vec![
                format!("{} ({})", row.name, row.kind),
                format!("{} of {} running", row.running, row.total),
            ];
            parts.extend(row.health.clone());
            parts.extend(row.urls.iter().cloned());
            parts.extend(row.problem.clone());
            text.push('\n');
            text.push_str(&parts.join(" · "));
        }
        text
    }
}

#[cfg(test)]
mod tests;
