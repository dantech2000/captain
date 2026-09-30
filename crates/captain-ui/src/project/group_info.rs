use std::collections::BTreeSet;

use captain_core::model::{Container, ContainerState, Health, count_label};
use captain_core::store::{ContainerGroup, GroupKey};
use gpui_kit::*;

use crate::icons::CaptainIcon;
use crate::theme::Palette;

/// What the sidebar and the Project page say about one entry.
pub struct GroupInfo {
    pub name: String,
    /// For example `Compose · 5 services`.
    pub kind: String,
    pub icon: CaptainIcon,
    /// The label color of a project, the action color of a namespace.
    pub color: Hsla,
    /// For example `4 of 5`, or `Stopped`.
    pub summary: String,
    /// The state color of the summary's dot.
    pub dot: Hsla,
    /// The host ports the containers publish, without duplicates.
    pub ports: Vec<u16>,
    pub help: String,
}

impl GroupInfo {
    /// `crashed` says if a container crashed lately, so a crash loop shows red even
    /// while the container runs between restarts.
    pub fn of(group: &ContainerGroup, crashed: &dyn Fn(&str) -> bool, palette: &Palette) -> Self {
        let shown: Vec<&Container> = group.containers.iter().filter(|c| !is_sandbox(c)).collect();
        let total = shown.len();
        let running = shown.iter().filter(|c| c.state.is_active()).count();
        let troubled = shown
            .iter()
            .filter(|c| crashed(&c.id) || needs_attention(c))
            .count();
        let (summary, dot) = match running {
            0 => ("Stopped".to_string(), palette.gray),
            _ if troubled > 0 => (format!("{running} of {total}"), palette.red),
            _ if running < total => (format!("{running} of {total}"), palette.orange),
            _ => (format!("{running} of {total}"), palette.green),
        };
        let attention = match troubled {
            0 => String::new(),
            1 => " 1 needs attention.".into(),
            n => format!(" {n} need attention."),
        };
        let mut ports: Vec<u16> = Vec::new();
        for port in shown.iter().flat_map(|c| c.published_ports()) {
            if !ports.contains(&port) {
                ports.push(port);
            }
        }
        let (name, kind, icon, color, help) = match &group.key {
            GroupKey::Project(name) => {
                let services: BTreeSet<&str> = shown
                    .iter()
                    .filter_map(|c| c.compose.service.as_deref())
                    .collect();
                let kind = format!("Compose · {}", count_label(services.len(), "service"));
                let help = format!(
                    "Show {name}: {running} of its {} run.{attention}",
                    count_label(total, "container")
                );
                (
                    name.clone(),
                    kind,
                    CaptainIcon::Stack,
                    palette.project_color(name),
                    help,
                )
            }
            GroupKey::Namespace(namespace) => {
                let pods: BTreeSet<String> = group
                    .containers
                    .iter()
                    .map(|c| pod_of(&c.display_name()))
                    .collect();
                let pods = count_label(pods.len(), "pod");
                let help = format!("Show the Kubernetes namespace {namespace}: {pods}.{attention}");
                let kind = format!("Kubernetes · {pods}");
                (
                    namespace.clone(),
                    kind,
                    CaptainIcon::Cluster,
                    palette.accent_fg,
                    help,
                )
            }
            GroupKey::Standalone => (
                "Loose containers".into(),
                "Not in a project".into(),
                CaptainIcon::Container,
                palette.text3,
                format!("Show the containers that are not in a Compose project.{attention}"),
            ),
        };
        Self {
            name,
            kind,
            icon,
            color,
            summary,
            dot,
            ports,
            help,
        }
    }
}

/// The name a card and the log use: the Compose service, else the shown name.
pub fn service_name(container: &Container) -> String {
    container
        .compose
        .service
        .clone()
        .unwrap_or_else(|| container.display_name())
}

/// True for the pause container that holds a Kubernetes pod's namespaces. It has no
/// output and nothing to act on, so the Project page leaves it out.
pub fn is_sandbox(container: &Container) -> bool {
    container.is_kubernetes() && container.name.starts_with("k8s_POD_")
}

/// True for a container that is restarting, dead, or failing its health check.
pub fn needs_attention(container: &Container) -> bool {
    matches!(
        container.state,
        ContainerState::Restarting | ContainerState::Dead
    ) || container.health == Some(Health::Unhealthy)
}

/// The pod part of `pod/container` or `pod (sandbox)`.
fn pod_of(display_name: &str) -> String {
    display_name
        .split(['/', ' '])
        .next()
        .unwrap_or_default()
        .to_string()
}
