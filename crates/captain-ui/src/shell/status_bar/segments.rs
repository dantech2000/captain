use captain_core::HostStatus;
use captain_core::format::{bytes_label, percent_label};
use captain_core::kubernetes::KubernetesStatus;
use gpui_kit::*;

use crate::engine_host::HostSummary;
use crate::theme::Palette;
use crate::workspace::{Connection, Workspace};

/// One item on the right of the status bar, with its own help sentence.
pub struct Segment {
    pub id: &'static str,
    pub label: SharedString,
    pub help: SharedString,
    pub dot: Option<Hsla>,
}

/// Engine, CPU, memory, Kubernetes, and context, left to right. CPU and memory show
/// only while connected. Disk comes with feature 0031.
pub fn segments(
    workspace: &Workspace,
    host: Option<&HostSummary>,
    kubernetes: Option<&KubernetesStatus>,
    context: Option<&str>,
    palette: &Palette,
) -> Vec<Segment> {
    let mut segments = vec![engine(workspace.connection(), host, palette)];
    if let Connection::Connected(info) = workspace.connection() {
        let stats = workspace.stats();
        let cpu = stats.total_cpu() / f64::from(info.cpus.max(1));
        segments.push(Segment {
            id: "status-cpu",
            label: format!("CPU {}", percent_label(cpu)).into(),
            help: format!(
                "CPU use of all containers, out of the engine's {} CPUs.",
                info.cpus
            )
            .into(),
            dot: None,
        });
        segments.push(Segment {
            id: "status-memory",
            label: format!(
                "Memory {} of {}",
                bytes_label(stats.total_memory()),
                bytes_label(info.memory_bytes)
            )
            .into(),
            help: "Memory use of all containers, out of what the engine has.".into(),
            dot: None,
        });
    }
    segments.extend(kubernetes.map(|status| kubernetes_segment(status, palette)));
    segments.extend(context.map(|name| Segment {
        id: "status-context",
        label: format!("context {name}").into(),
        help: format!("The docker CLI in your terminals uses the context {name}.").into(),
        dot: None,
    }));
    segments
}

fn engine(connection: &Connection, host: Option<&HostSummary>, palette: &Palette) -> Segment {
    if let Some(host) = host {
        let (dot, help) = match &host.status {
            HostStatus::Running => (
                palette.green,
                "Captain Engine is running. Start or stop it in the engine card.".to_string(),
            ),
            HostStatus::Starting | HostStatus::Stopping => (
                palette.orange,
                format!("Captain Engine is {}.", host.status.label().to_lowercase()),
            ),
            HostStatus::NotCreated => (
                palette.gray,
                "Captain Engine is not set up. Set it up in the engine card.".to_string(),
            ),
            HostStatus::Failed(why) | HostStatus::NotInstalled(why) => {
                (palette.red, format!("Captain Engine cannot run: {why}"))
            }
            HostStatus::Stopped => (
                palette.red,
                "Captain Engine is stopped. Start it in the engine card.".to_string(),
            ),
        };
        return Segment {
            id: "status-engine",
            label: "Captain Engine".into(),
            help: help.into(),
            dot: Some(dot),
        };
    }
    let (label, dot, help) = match connection {
        Connection::Connecting => (
            "Engine".to_string(),
            palette.orange,
            "Captain is connecting to the engine.".to_string(),
        ),
        Connection::Connected(info) => {
            let name = engine_name(&info.endpoint);
            let help = format!("Captain uses {name} at {}.", info.endpoint);
            (name.to_string(), palette.green, help)
        }
        Connection::Failed(error) => (
            "Engine".to_string(),
            palette.red,
            format!("Captain cannot reach the engine: {error}"),
        ),
    };
    Segment {
        id: "status-engine",
        label: label.into(),
        help: help.into(),
        dot: Some(dot),
    }
}

fn kubernetes_segment(status: &KubernetesStatus, palette: &Palette) -> Segment {
    let (label, dot, help) = match status {
        KubernetesStatus::Off => (
            "Kubernetes off".to_string(),
            palette.gray,
            "Kubernetes is off. Turn it on in Settings > Kubernetes.".to_string(),
        ),
        KubernetesStatus::Starting => (
            "Kubernetes starting".to_string(),
            palette.orange,
            "Kubernetes is starting in Captain Engine.".to_string(),
        ),
        KubernetesStatus::Running { version } => (
            "Kubernetes on".to_string(),
            palette.green,
            format!("Kubernetes {version} runs in Captain Engine."),
        ),
        KubernetesStatus::Failed(why) => (
            "Kubernetes failed".to_string(),
            palette.red,
            format!("Kubernetes did not start: {why}"),
        ),
    };
    Segment {
        id: "status-kubernetes",
        label: label.into(),
        help: help.into(),
        dot: Some(dot),
    }
}

/// A name for an engine that is not Captain Engine, from its socket path.
fn engine_name(endpoint: &str) -> &'static str {
    const KNOWN: [(&str, &str); 5] = [
        (".docker/run/", "Docker Desktop"),
        (".orbstack/", "OrbStack"),
        (".colima/", "Colima"),
        (".rd/", "Rancher Desktop"),
        ("podman", "Podman"),
    ];
    KNOWN
        .iter()
        .find(|(fragment, _)| endpoint.contains(fragment))
        .map(|(_, name)| *name)
        .unwrap_or(
            if endpoint.starts_with("unix://") || endpoint.starts_with("npipe://") {
                "Docker Engine"
            } else {
                "Remote engine"
            },
        )
}
