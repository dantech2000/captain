//! The Projects submenu: one submenu per Compose project, with its status dot.

use std::collections::BTreeMap;

use captain_core::model::ContainerAction;

use super::dot::Light;
use super::entries::ContainerEntry;
use super::menu_model::{TrayCommand, TrayItem};
use super::snapshot::TraySnapshot;

/// The projects in name order.
pub fn projects(snapshot: &TraySnapshot) -> Vec<TrayItem> {
    let mut groups: BTreeMap<&str, Vec<&ContainerEntry>> = BTreeMap::new();
    for entry in &snapshot.containers {
        if let Some(project) = entry.project.as_deref() {
            groups.entry(project).or_default().push(entry);
        }
    }
    if groups.is_empty() {
        return vec![TrayItem::Label("No Compose Projects".into())];
    }
    groups
        .into_iter()
        .map(|(name, entries)| project(name, &entries))
        .collect()
}

/// Start, Stop, and Restart for the project's containers.
fn project(name: &str, entries: &[&ContainerEntry]) -> TrayItem {
    let ids = |keep: fn(&ContainerEntry) -> bool| -> Vec<String> {
        entries
            .iter()
            .filter(|e| keep(e))
            .map(|e| e.id.clone())
            .collect()
    };
    // Start only what is stopped and stop only what runs, so the engine does not
    // report "already started" as a failure.
    let item = |label: &str, ids: Vec<String>, action| TrayItem::Command {
        label: label.into(),
        enabled: !ids.is_empty(),
        command: TrayCommand::Containers { ids, action },
    };
    let active = entries.iter().filter(|e| e.is_active()).count();
    TrayItem::Submenu {
        label: format!("{name} ({active}/{})", entries.len()),
        light: Some(light(entries, active)),
        items: vec![
            item("Start", ids(|e| !e.is_active()), ContainerAction::Start),
            item(
                "Stop",
                ids(ContainerEntry::is_active),
                ContainerAction::Stop,
            ),
            item("Restart", ids(|_| true), ContainerAction::Restart),
        ],
    }
}

/// Red when a container fails, green when all run, amber when some run, else gray.
fn light(entries: &[&ContainerEntry], active: usize) -> Light {
    if entries.iter().any(|e| e.failing) {
        Light::Red
    } else if active == entries.len() {
        Light::Green
    } else if active > 0 {
        Light::Amber
    } else {
        Light::Gray
    }
}

#[cfg(test)]
mod tests;
