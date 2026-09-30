//! The Containers and Open Ports submenus.

use captain_core::model::{ContainerAction, ContainerState, PortLink};

use super::entries::ContainerEntry;
use super::menu_model::{TrayCommand, TrayItem};
use super::snapshot::TraySnapshot;

/// One submenu per running container, with its status dot.
pub fn containers(snapshot: &TraySnapshot) -> Vec<TrayItem> {
    let items: Vec<TrayItem> = snapshot
        .containers
        .iter()
        .filter(|c| c.is_active())
        .map(container)
        .collect();
    if items.is_empty() {
        return vec![TrayItem::Label("No Running Containers".into())];
    }
    items
}

/// Stop, Restart, Show Logs, and its published ports.
fn container(entry: &ContainerEntry) -> TrayItem {
    let action = |action| TrayCommand::Container {
        id: entry.id.clone(),
        action,
    };
    let mut items = vec![
        TrayItem::command("Stop", action(ContainerAction::Stop)),
        TrayItem::command("Restart", action(ContainerAction::Restart)),
        TrayItem::command(
            "Show Logs in a Window",
            TrayCommand::FloatLog {
                id: entry.id.clone(),
                name: entry.name.clone(),
            },
        ),
    ];
    if !entry.ports.is_empty() {
        items.push(TrayItem::Separator);
        items.extend(entry.ports.iter().map(|link| {
            let label = match link {
                PortLink::Open(url) => format!("Open {} in Browser", address(url)),
                PortLink::Copy { address, .. } => format!("Copy {address}"),
            };
            TrayItem::command(label, port_command(link))
        }));
    }
    TrayItem::Submenu {
        label: entry.name.clone(),
        light: Some(entry.light()),
        items,
    }
}

/// Every port that a running container publishes, once, with its service: a web
/// port opens in the browser, a database port copies its address, as on the Project
/// page.
pub fn open_ports(snapshot: &TraySnapshot) -> Vec<TrayItem> {
    let mut seen = Vec::new();
    let mut items = Vec::new();
    for entry in &snapshot.containers {
        if entry.state != ContainerState::Running {
            continue;
        }
        for link in &entry.ports {
            if seen.contains(&link) {
                continue;
            }
            seen.push(link);
            let label = match link {
                PortLink::Open(url) => format!("{} \u{2014} {}", entry.service, address(url)),
                PortLink::Copy { address, .. } => {
                    format!("{} \u{2014} Copy {address}", entry.service)
                }
            };
            items.push(TrayItem::command(label, port_command(link)));
        }
    }
    if items.is_empty() {
        return vec![TrayItem::Label("No Open Ports".into())];
    }
    items
}

fn port_command(link: &PortLink) -> TrayCommand {
    match link {
        PortLink::Open(url) => TrayCommand::OpenUrl(url.clone()),
        PortLink::Copy { address, .. } => TrayCommand::CopyAddress(address.clone()),
    }
}

/// `localhost:8080` for `http://localhost:8080`.
fn address(url: &str) -> &str {
    url.strip_prefix("http://").unwrap_or(url)
}

#[cfg(test)]
mod tests;
