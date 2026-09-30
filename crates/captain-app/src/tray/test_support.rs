//! Shared builders for the menu tests.

use captain_core::model::ContainerState;

use super::dot::Light;
use super::entries::ContainerEntry;
use super::menu_model::TrayItem;
use super::snapshot::{EngineStatus, TraySnapshot};

pub fn entry(name: &str, state: ContainerState, project: Option<&str>) -> ContainerEntry {
    ContainerEntry {
        id: format!("{name}-id"),
        name: name.into(),
        service: name.into(),
        state,
        failing: false,
        project: project.map(Into::into),
        ports: Vec::new(),
    }
}

pub fn snapshot(containers: Vec<ContainerEntry>) -> TraySnapshot {
    TraySnapshot {
        engine: EngineStatus::Running,
        status_line: "Captain Engine: Running".into(),
        containers,
        host: None,
        kubernetes: None,
        contexts: Default::default(),
        problem: None,
    }
}

pub fn labels(items: &[TrayItem]) -> Vec<String> {
    items
        .iter()
        .map(|item| match item {
            TrayItem::Label(label)
            | TrayItem::Status { label, .. }
            | TrayItem::Command { label, .. }
            | TrayItem::Check { label, .. }
            | TrayItem::Submenu { label, .. } => label.clone(),
            TrayItem::Separator => "-".into(),
        })
        .collect()
}

pub fn find<'a>(items: &'a [TrayItem], name: &str) -> &'a TrayItem {
    items
        .iter()
        .find(|item| labels(std::slice::from_ref(item))[0] == name)
        .unwrap_or_else(|| panic!("no item {name}"))
}

pub fn submenu<'a>(items: &'a [TrayItem], name: &str) -> &'a [TrayItem] {
    match find(items, name) {
        TrayItem::Submenu { items, .. } => items,
        other => panic!("{name} is no submenu: {other:?}"),
    }
}

pub fn light(item: &TrayItem) -> Option<Light> {
    match item {
        TrayItem::Status { light, .. } => Some(*light),
        TrayItem::Submenu { light, .. } => *light,
        _ => None,
    }
}
