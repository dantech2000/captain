use std::collections::BTreeMap;

use captain_core::model::ContainerAction;

use captain_core::HostStatus;

use super::snapshot::{ContainerEntry, EngineStatus, HostEntry, TraySnapshot};

/// What a menu item does when the user picks it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrayCommand {
    OpenCaptain,
    Settings,
    Quit,
    /// Starts Captain Engine.
    StartEngine,
    /// Stops Captain Engine.
    StopEngine,
    /// Runs an action on one container.
    Container {
        id: String,
        action: ContainerAction,
    },
    /// Runs an action on each container of a Compose project.
    Project {
        ids: Vec<String>,
        action: ContainerAction,
    },
    /// Opens `http://localhost:<port>` in the browser.
    OpenPort(u16),
}

/// The menu as plain data, so it can be tested without a menu bar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrayItem {
    /// A line of text that cannot be clicked.
    Label(String),
    Command {
        label: String,
        command: TrayCommand,
        enabled: bool,
    },
    Submenu {
        label: String,
        items: Vec<TrayItem>,
    },
    Separator,
}

impl TrayItem {
    fn command(label: impl Into<String>, command: TrayCommand) -> Self {
        Self::Command {
            label: label.into(),
            command,
            enabled: true,
        }
    }
}

/// The whole menu for `snapshot`, top to bottom.
pub fn build(snapshot: &TraySnapshot) -> Vec<TrayItem> {
    let status = snapshot
        .host
        .as_ref()
        .map_or(snapshot.engine.label(), HostEntry::label);
    let mut items = vec![TrayItem::Label(status.into())];
    let running = snapshot.engine == EngineStatus::Running;
    if running {
        items.push(TrayItem::Label(count_label(snapshot)));
    }
    items.extend(snapshot.host.as_ref().and_then(host_item));
    items.extend([
        TrayItem::Separator,
        TrayItem::command("Open Captain", TrayCommand::OpenCaptain),
        TrayItem::command("Settings\u{2026}", TrayCommand::Settings),
    ]);
    if running {
        items.extend([
            TrayItem::Separator,
            TrayItem::Submenu {
                label: "Containers".into(),
                items: containers(snapshot),
            },
            TrayItem::Submenu {
                label: "Projects".into(),
                items: projects(snapshot),
            },
        ]);
    }
    items.extend([
        TrayItem::Separator,
        TrayItem::command("Quit Captain", TrayCommand::Quit),
    ]);
    items
}

/// Start or Stop for Captain Engine. Setup has choices, so it opens the window.
fn host_item(host: &HostEntry) -> Option<TrayItem> {
    if !host.can_control {
        return None;
    }
    let (label, command, enabled) = match host.status {
        HostStatus::Running | HostStatus::Starting => {
            ("Stop Captain Engine", TrayCommand::StopEngine, true)
        }
        HostStatus::Stopping => ("Stop Captain Engine", TrayCommand::StopEngine, false),
        HostStatus::NotCreated => (
            "Set Up Captain Engine\u{2026}",
            TrayCommand::OpenCaptain,
            true,
        ),
        HostStatus::NotInstalled(_) => ("Start Captain Engine", TrayCommand::StartEngine, false),
        HostStatus::Stopped | HostStatus::Failed(_) => {
            ("Start Captain Engine", TrayCommand::StartEngine, true)
        }
    };
    Some(TrayItem::Command {
        label: label.into(),
        command,
        enabled,
    })
}

/// "3 of 5 containers running".
fn count_label(snapshot: &TraySnapshot) -> String {
    let total = snapshot.containers.len();
    let noun = if total == 1 {
        "container"
    } else {
        "containers"
    };
    format!("{} of {total} {noun} running", snapshot.active_count())
}

/// One submenu per running container: Stop, Restart, and its published ports.
fn containers(snapshot: &TraySnapshot) -> Vec<TrayItem> {
    let items: Vec<TrayItem> = snapshot
        .containers
        .iter()
        .filter(|c| c.is_active())
        .map(container)
        .collect();
    if items.is_empty() {
        return vec![TrayItem::Label("No running containers".into())];
    }
    items
}

fn container(entry: &ContainerEntry) -> TrayItem {
    let action = |action| TrayCommand::Container {
        id: entry.id.clone(),
        action,
    };
    let mut items = vec![
        TrayItem::command("Stop", action(ContainerAction::Stop)),
        TrayItem::command("Restart", action(ContainerAction::Restart)),
    ];
    if !entry.ports.is_empty() {
        items.push(TrayItem::Separator);
        items.extend(entry.ports.iter().map(|&port| {
            TrayItem::command(
                format!("Open localhost:{port} in browser"),
                TrayCommand::OpenPort(port),
            )
        }));
    }
    TrayItem::Submenu {
        label: entry.name.clone(),
        items,
    }
}

/// One submenu per Compose project, in name order: Start all, Stop all, Restart all.
fn projects(snapshot: &TraySnapshot) -> Vec<TrayItem> {
    let mut groups: BTreeMap<&str, Vec<&ContainerEntry>> = BTreeMap::new();
    for entry in &snapshot.containers {
        if let Some(project) = entry.project.as_deref() {
            groups.entry(project).or_default().push(entry);
        }
    }
    if groups.is_empty() {
        return vec![TrayItem::Label("No Compose projects".into())];
    }
    groups
        .into_iter()
        .map(|(name, entries)| project(name, &entries))
        .collect()
}

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
        command: TrayCommand::Project { ids, action },
    };
    let active = entries.iter().filter(|e| e.is_active()).count();
    TrayItem::Submenu {
        label: format!("{name} ({active}/{})", entries.len()),
        items: vec![
            item("Start all", ids(|e| !e.is_active()), ContainerAction::Start),
            item(
                "Stop all",
                ids(ContainerEntry::is_active),
                ContainerAction::Stop,
            ),
            item("Restart all", ids(|_| true), ContainerAction::Restart),
        ],
    }
}

#[cfg(test)]
mod tests;
