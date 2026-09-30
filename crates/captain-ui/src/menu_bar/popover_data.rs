//! What the popover shows, gathered once per render from the workspace, Captain
//! Engine, Kubernetes, and the diagnostics checks.

use std::collections::{BTreeSet, HashMap};

use captain_core::HostStatus;
use captain_core::format::bytes_label;
use captain_core::kubernetes::KubernetesStatus;
use captain_core::model::{ContainerState, ProjectAction};
use captain_core::problems::{ExitFacts, Problem, first_problem};
use gpui_kit::*;

use crate::diagnostics::diagnostics_model;
use crate::engine_host::summary as host_summary;
use crate::kubernetes::kubernetes_model;
use crate::shell::engine_name;
use crate::workspace::{Connection, Workspace};

/// The most ports the popover lists.
const MAX_PORTS: usize = 8;

/// Ports of databases and caches. A browser cannot open them, so a click copies.
const DATABASE_PORTS: [u16; 8] = [3306, 5432, 6379, 27017, 9042, 11211, 1433, 5984];

/// The engine header.
pub struct EngineLine {
    pub name: &'static str,
    pub line: String,
    /// On while the engine runs or starts.
    pub on: bool,
    /// False when Captain cannot start or stop this engine now.
    pub can_switch: bool,
    /// Captain Engine's status, `None` for another engine.
    pub host: Option<HostStatus>,
}

/// One Compose project row.
pub struct ProjectRow {
    pub name: String,
    pub ids: Vec<String>,
    pub active: usize,
    pub total: usize,
    pub pending: Option<ProjectAction>,
}

/// One published port.
pub struct PortRow {
    pub service: String,
    pub port: u16,
    pub copies: bool,
}

pub struct PopoverData {
    pub engine: EngineLine,
    pub problem: Option<Problem>,
    pub projects: Vec<ProjectRow>,
    /// Kubernetes' status and whether the settings turn it on, when it shows.
    pub kubernetes: Option<(KubernetesStatus, bool)>,
    pub ports: Vec<PortRow>,
    /// The container that "Float logs" opens: the problem's, else the selected one.
    pub float: Option<(String, String)>,
    /// Every active container, for Stop all.
    pub active: Vec<String>,
}

impl PopoverData {
    pub fn gather(workspace: &Workspace, facts: &HashMap<String, ExitFacts>, cx: &App) -> Self {
        let host = host_summary(cx);
        let engine = engine_line(workspace, host.as_ref().map(|h| (&h.status, h.can_control)));
        let connected = matches!(workspace.connection(), Connection::Connected(_));
        let containers: &[_] = if connected {
            workspace.store().containers()
        } else {
            &[]
        };
        let checks = diagnostics_model(cx)
            .map(|model| model.read(cx).checks().to_vec())
            .unwrap_or_default();
        let failure = host.as_ref().and_then(|h| match &h.status {
            HostStatus::Failed(why) => Some(why.as_str()),
            _ => None,
        });
        let problem = first_problem(failure, &checks, containers, facts);
        let float = problem
            .as_ref()
            .and_then(Problem::container)
            .map(|(id, name)| (id.to_string(), name.to_string()))
            .or_else(|| {
                workspace
                    .selected()
                    .filter(|c| c.state.is_active())
                    .map(|c| (c.id.clone(), c.name.clone()))
            });
        let projects = if connected {
            projects(workspace)
        } else {
            Vec::new()
        };
        let kubernetes = host.as_ref().and(kubernetes_model(cx)).and_then(|model| {
            let model = model.read(cx);
            let enabled = model.settings(cx).enabled;
            let status = model.status().clone();
            (enabled || status != KubernetesStatus::Off).then_some((status, enabled))
        });
        Self {
            engine,
            problem,
            projects,
            kubernetes,
            ports: ports(containers),
            float,
            active: containers
                .iter()
                .filter(|c| c.state.is_active())
                .map(|c| c.id.clone())
                .collect(),
        }
    }

    /// The popover's height for this content, before the screen limits it.
    pub fn height(&self) -> Pixels {
        let mut height = 64. + 58. + 30.;
        if self.problem.is_some() {
            height += 112.;
        }
        let rows = self.projects.len() + usize::from(self.kubernetes.is_some());
        if rows > 0 {
            height += 26. + 38. * rows as f32;
        }
        if !self.ports.is_empty() {
            height += 34. + 30. * self.ports.len() as f32;
        }
        px(height)
    }
}

fn engine_line(workspace: &Workspace, host: Option<(&HostStatus, bool)>) -> EngineLine {
    let usage = match workspace.connection() {
        Connection::Connected(info) => Some(format!(
            "Running · {} CPUs · {} of {}",
            info.cpus,
            bytes_label(workspace.stats().total_memory()),
            bytes_label(info.memory_bytes)
        )),
        _ => None,
    };
    if let Some((status, can_control)) = host {
        let line = match status {
            HostStatus::Running => usage.unwrap_or_else(|| "Connecting".into()),
            HostStatus::Failed(_) => "Did not start".into(),
            status => status.label().into(),
        };
        let on = matches!(status, HostStatus::Running | HostStatus::Starting);
        let settled = !matches!(status, HostStatus::Starting | HostStatus::Stopping);
        return EngineLine {
            name: "Captain Engine",
            line,
            on,
            can_switch: can_control && settled,
            host: Some(status.clone()),
        };
    }
    let (name, line) = match workspace.connection() {
        Connection::Connected(info) => (engine_name(&info.endpoint), usage.unwrap_or_default()),
        Connection::Connecting => ("Engine", "Connecting".into()),
        Connection::Failed(_) => ("Engine", "Not reachable".into()),
    };
    EngineLine {
        name,
        line,
        on: matches!(workspace.connection(), Connection::Connected(_)),
        can_switch: false,
        host: None,
    }
}

fn projects(workspace: &Workspace) -> Vec<ProjectRow> {
    workspace
        .compose_projects()
        .into_iter()
        .map(|project| ProjectRow {
            ids: project.containers().map(|c| c.id.clone()).collect(),
            active: project.active_count(),
            total: project.container_count(),
            pending: workspace.project_pending(&project.name),
            name: project.name,
        })
        .collect()
}

fn ports(containers: &[captain_core::model::Container]) -> Vec<PortRow> {
    let mut seen = BTreeSet::new();
    containers
        .iter()
        .filter(|c| c.state == ContainerState::Running)
        .flat_map(|c| {
            let service = c.compose.service.clone().unwrap_or_else(|| c.name.clone());
            c.published_ports()
                .into_iter()
                .map(move |port| (service.clone(), port))
        })
        .filter(|(_, port)| seen.insert(*port))
        .take(MAX_PORTS)
        .map(|(service, port)| PortRow {
            service,
            port,
            copies: DATABASE_PORTS.contains(&port),
        })
        .collect()
}
