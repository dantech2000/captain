use gpui_kit::*;

use crate::engine_host::HostSummary;
use crate::shell::status_bar::engine_name;
use crate::theme::Palette;
use crate::workspace::{Connection, Workspace};

/// Which engine the window talks to and its state, as the sidebar header and the
/// dot on the rail's app icon show them.
pub struct EngineState {
    /// For example `Captain Engine`.
    pub engine: &'static str,
    /// For example `Running`.
    pub state: &'static str,
    pub color: Hsla,
}

impl EngineState {
    /// With Captain Engine the state comes from the host, except that a running
    /// engine that does not answer shows red.
    pub fn of(workspace: &Workspace, host: Option<&HostSummary>, palette: &Palette) -> Self {
        let connection = workspace.connection();
        let (color, state) = match (host, connection) {
            (Some(host), Connection::Failed(_)) if host.status.is_running() => {
                (palette.red, "Not answering")
            }
            (Some(host), _) => (palette.host_status(&host.status), host.status.label()),
            (None, Connection::Connecting) => (palette.orange, "Connecting"),
            (None, Connection::Connected(_)) => (palette.green, "Running"),
            (None, Connection::Failed(_)) => (palette.red, "Not answering"),
        };
        let engine = match (host, connection) {
            (Some(_), _) => "Captain Engine",
            (None, Connection::Connected(info)) => engine_name(&info.endpoint),
            (None, _) => "Engine",
        };
        Self {
            engine,
            state,
            color,
        }
    }

    /// For example `Captain Engine · Running`.
    pub fn line(&self) -> String {
        format!("{} · {}", self.engine, self.state)
    }

    /// The status bar sentence for the engine header and the rail's app icon.
    pub fn help(&self) -> String {
        format!(
            "{}: {}. The line at the bottom of the sidebar shows its CPU, memory, and disk.",
            self.engine,
            self.state.to_lowercase()
        )
    }
}
