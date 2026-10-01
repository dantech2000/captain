use captain_core::HostStatus;
use captain_core::agent_clients::client_label;
use captain_core::agent_tools::Activity;
use captain_core::format::{bytes_label, percent_label};
use captain_core::kubernetes::KubernetesStatus;
use gpui_kit::*;

use crate::engine_host::HostSummary;
use crate::theme::Palette;
use crate::workspace::{Connection, Page, Workspace};

/// One item on the right of the status bar, with its own help sentence.
pub struct Segment {
    pub id: &'static str,
    pub label: SharedString,
    pub help: SharedString,
    pub dot: Option<Hsla>,
    /// The label's color, when it is not the bar's.
    pub color: Option<Hsla>,
    /// The page a click opens.
    pub page: Option<Page>,
}

/// Engine, CPU, memory, Kubernetes, and context, left to right. CPU and memory show
/// only while connected. The Disk segment comes from [`disk`].
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
        let cpus = match info.cpus {
            1 => "1 CPU".to_string(),
            n => format!("{n} CPUs"),
        };
        let memory = format!(
            "{} of {}",
            bytes_label(stats.total_memory()),
            bytes_label(info.memory_bytes)
        );
        segments.push(Segment {
            id: "status-cpu",
            label: format!("CPU {}", percent_label(cpu)).into(),
            help: format!(
                "CPU: {} of {cpus}, used by all containers.",
                percent_label(cpu)
            )
            .into(),
            dot: None,
            color: None,
            page: None,
        });
        segments.push(Segment {
            id: "status-memory",
            label: format!("Memory {memory}").into(),
            help: format!("Memory: {memory}, used by all containers.").into(),
            dot: None,
            color: None,
            page: None,
        });
    }
    segments.extend(kubernetes.map(|status| kubernetes_segment(status, palette)));
    segments.extend(context.map(|name| Segment {
        id: "status-context",
        label: format!("context {name}").into(),
        help: format!("The docker CLI in your terminals uses the context {name}.").into(),
        dot: None,
        color: None,
        page: None,
    }));
    segments
}

/// The latest agent call, such as "Claude Code: restart api", for ten minutes.
/// See feature 0038.
pub fn agent(entry: &Activity, time: &str, palette: &Palette) -> Segment {
    let client = client_label(&entry.client);
    let dot = match (entry.ok, entry.is_action()) {
        (false, _) => palette.red,
        (true, true) => palette.orange,
        (true, false) => palette.gray,
    };
    let outcome = match (entry.ok, entry.result.is_empty()) {
        (true, _) | (false, true) => String::new(),
        (false, false) => format!(" It failed: {}", entry.result),
    };
    Segment {
        id: "status-agent",
        label: format!("{client}: {}", entry.tool).into(),
        help: format!(
            "{client} called {} at {time}.{outcome} Every call is in Settings > AI agents.",
            entry.summary()
        )
        .into(),
        dot: Some(dot),
        color: None,
        page: None,
    }
}

/// "Disk 18.2 GB of 64 GB", or without the size for an engine Captain does not run.
/// It turns the warning text color when a cleanup can free some, and a click opens
/// Storage. See feature 0031.
pub fn disk(used: u64, capacity: Option<u64>, freeable: u64, palette: &Palette) -> Segment {
    let (label, used) = match capacity {
        Some(capacity) => {
            let size = format!("{} of {}", bytes_label(used), bytes_label(capacity));
            (format!("Disk {size}"), size)
        }
        None => (
            format!("Disk {}", bytes_label(used)),
            format!("{} used", bytes_label(used)),
        ),
    };
    let (color, help) = if freeable > 0 {
        (
            Some(palette.warn_text),
            format!(
                "Disk: {used}. {} can be freed. Click to review.",
                bytes_label(freeable)
            ),
        )
    } else {
        (None, format!("Disk: {used}. Click to open Storage."))
    };
    Segment {
        id: "status-disk",
        label: label.into(),
        help: help.into(),
        dot: None,
        color,
        page: Some(Page::Storage),
    }
}

/// The end of the engine segment's sentence: a click opens Diagnostics, whose Engine
/// card has Start, Stop, and Restart.
const CONTROLS: &str = "Click to start, stop, or restart it on the Diagnostics page.";
const DIAGNOSTICS: &str = "Click to open Diagnostics.";

fn engine(connection: &Connection, host: Option<&HostSummary>, palette: &Palette) -> Segment {
    if let Some(host) = host {
        let (dot, help) = match &host.status {
            HostStatus::Running => (
                palette.green,
                format!("Captain Engine is running. {CONTROLS}"),
            ),
            HostStatus::Starting | HostStatus::Stopping => (
                palette.orange,
                format!(
                    "Captain Engine is {}. {CONTROLS}",
                    host.status.label().to_lowercase()
                ),
            ),
            HostStatus::NotCreated => (
                palette.gray,
                "Captain Engine is not set up. Click to set it up on the Diagnostics page."
                    .to_string(),
            ),
            HostStatus::Failed(why) | HostStatus::NotInstalled(why) => (
                palette.red,
                format!("Captain Engine cannot run: {why} {DIAGNOSTICS}"),
            ),
            HostStatus::Stopped => (
                palette.red,
                format!("Captain Engine is stopped. {CONTROLS}"),
            ),
        };
        return Segment {
            id: "status-engine",
            label: "Captain Engine".into(),
            help: help.into(),
            dot: Some(dot),
            color: None,
            page: Some(Page::Diagnostics),
        };
    }
    let (label, dot, help) = match connection {
        Connection::Connecting => (
            "Engine".to_string(),
            palette.orange,
            format!("Captain is connecting to the engine. {DIAGNOSTICS}"),
        ),
        Connection::Connected(info) => {
            let name = engine_name(&info.endpoint);
            let help = format!("Captain uses {name} at {}. {DIAGNOSTICS}", info.endpoint);
            (name.to_string(), palette.green, help)
        }
        Connection::Failed(error) => (
            "Engine".to_string(),
            palette.red,
            format!("Captain cannot reach the engine: {error} {DIAGNOSTICS}"),
        ),
    };
    Segment {
        id: "status-engine",
        label: label.into(),
        help: help.into(),
        dot: Some(dot),
        color: None,
        page: Some(Page::Diagnostics),
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
        color: None,
        page: None,
    }
}

/// A name for an engine that is not Captain Engine, from its socket path.
pub fn engine_name(endpoint: &str) -> &'static str {
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
