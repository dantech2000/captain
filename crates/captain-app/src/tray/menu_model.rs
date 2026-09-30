//! The native menu as plain data, top to bottom. It follows the macOS menu style:
//! Title Case items, "…" on items that open a window or ask, disabled lines for
//! status, separators between groups, and submenus for lists. See feature 0009.

use captain_core::HostStatus;
use captain_core::diagnostics::Fix;
use captain_core::model::ContainerAction;

use super::containers_menu::{containers, open_ports};
use super::dot::Light;
use super::entries::HostEntry;
use super::kubernetes_menu::kubernetes_items;
use super::problem_menu::problem_items;
use super::projects_menu::projects;
use super::snapshot::{EngineStatus, TraySnapshot};

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
    /// Runs an action on each of several containers.
    Containers {
        ids: Vec<String>,
        action: ContainerAction,
    },
    /// Opens the container's log in a small window that stays on top.
    FloatLog {
        id: String,
        name: String,
    },
    /// Sets the container's memory limit to `bytes`.
    RaiseMemory {
        id: String,
        name: String,
        bytes: u64,
    },
    /// Runs the fix that Diagnostics suggests.
    RunFix(Fix),
    /// Opens a web page in the browser.
    OpenUrl(String),
    /// Copies an address, such as `localhost:5432`.
    CopyAddress(String),
    /// Turns Kubernetes on or off.
    SetKubernetes(bool),
    /// Makes a Kubernetes context the current one.
    UseContext(String),
}

/// The menu as plain data, so it can be tested without a menu bar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrayItem {
    /// A line of text that cannot be clicked.
    Label(String),
    /// A status line that cannot be clicked, with a colored dot.
    Status {
        label: String,
        light: Light,
    },
    Command {
        label: String,
        command: TrayCommand,
        enabled: bool,
    },
    /// A command with a check mark, such as the current Kubernetes context.
    Check {
        label: String,
        command: TrayCommand,
        checked: bool,
    },
    /// A submenu, with a colored dot for a container or a project.
    Submenu {
        label: String,
        light: Option<Light>,
        items: Vec<TrayItem>,
    },
    Separator,
}

impl TrayItem {
    pub fn command(label: impl Into<String>, command: TrayCommand) -> Self {
        Self::Command {
            label: label.into(),
            command,
            enabled: true,
        }
    }

    pub fn submenu(label: impl Into<String>, items: Vec<TrayItem>) -> Self {
        Self::Submenu {
            label: label.into(),
            light: None,
            items,
        }
    }
}

/// The whole menu for `snapshot`, top to bottom.
pub fn build(snapshot: &TraySnapshot) -> Vec<TrayItem> {
    let mut items = vec![TrayItem::Status {
        label: snapshot.status_line.clone(),
        light: engine_light(snapshot.engine),
    }];
    let running = snapshot.engine == EngineStatus::Running;
    if running {
        items.push(TrayItem::Label(count_label(snapshot)));
    }
    if let Some(problem) = &snapshot.problem {
        items.push(TrayItem::Separator);
        items.extend(problem_items(problem));
    }
    items.push(TrayItem::Separator);
    items.extend(snapshot.host.as_ref().and_then(host_item));
    items.extend([
        TrayItem::command("Open Captain", TrayCommand::OpenCaptain),
        TrayItem::command("Settings\u{2026}", TrayCommand::Settings),
    ]);
    if running {
        items.extend([
            TrayItem::Separator,
            TrayItem::submenu("Containers", containers(snapshot)),
            TrayItem::submenu("Projects", projects(snapshot)),
            TrayItem::submenu("Open Ports", open_ports(snapshot)),
        ]);
    }
    let kubernetes = kubernetes_items(snapshot);
    if !kubernetes.is_empty() {
        items.push(TrayItem::Separator);
        items.extend(kubernetes);
    }
    if running {
        items.extend([TrayItem::Separator, stop_all(snapshot)]);
    }
    items.extend([
        TrayItem::Separator,
        TrayItem::command("Quit Captain", TrayCommand::Quit),
    ]);
    items
}

fn engine_light(engine: EngineStatus) -> Light {
    match engine {
        EngineStatus::Running => Light::Green,
        EngineStatus::Starting => Light::Amber,
        EngineStatus::Stopped => Light::Gray,
        EngineStatus::NeedsAttention => Light::Red,
    }
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

/// Stops every running container. Nothing is removed.
fn stop_all(snapshot: &TraySnapshot) -> TrayItem {
    let ids: Vec<String> = snapshot
        .containers
        .iter()
        .filter(|c| c.is_active())
        .map(|c| c.id.clone())
        .collect();
    TrayItem::Command {
        label: "Stop All Containers".into(),
        enabled: !ids.is_empty(),
        command: TrayCommand::Containers {
            ids,
            action: ContainerAction::Stop,
        },
    }
}

#[cfg(test)]
mod tests;
