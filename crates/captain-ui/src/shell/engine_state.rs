use gpui_kit::*;

use crate::engine_host::HostSummary;
use crate::shell::status_bar::engine_name;
use crate::theme::Palette;
use crate::workspace::{Connection, EngineHealth, Page, Workspace};

/// Which engine the window talks to and its state, for the Diagnostics Engine card
/// and the help sentence of the rail's app icon.
pub struct EngineState {
    /// For example `Captain Engine`.
    pub engine: &'static str,
    /// For example `Running`.
    pub state: &'static str,
    pub color: Hsla,
    /// True for Captain Engine, which Captain can start and stop.
    pub captain: bool,
}

impl EngineState {
    /// With Captain Engine the state comes from the host, except that a running
    /// engine that does not answer shows "Reconnecting" in orange while Captain
    /// reconnects by itself, then red.
    pub fn of(workspace: &Workspace, host: Option<&HostSummary>, palette: &Palette) -> Self {
        let connection = workspace.connection();
        let health = workspace.engine_health(host.map(|host| &host.status));
        let (color, state) = match (health, host) {
            (EngineHealth::Reconnecting, _) => (palette.orange, "Reconnecting"),
            (EngineHealth::NotAnswering, _) => (palette.red, "Not answering"),
            (_, Some(host)) => (palette.host_status(&host.status), host.status.label()),
            (EngineHealth::Running, None) => (palette.green, "Running"),
            (_, None) => (palette.orange, "Connecting"),
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
            captain: host.is_some(),
        }
    }

    /// The status bar sentence for the rail's app icon, which opens Diagnostics on a
    /// click.
    pub fn help(&self) -> String {
        let click = if self.captain {
            "Click to start, stop, or restart it on the Diagnostics page."
        } else {
            "Click to open Diagnostics."
        };
        format!("{}: {}. {click}", self.engine, self.state.to_lowercase())
    }
}

/// A click handler that opens the Diagnostics page, where the Engine card has the
/// engine's controls. See feature 0016.
pub fn open_diagnostics(
    handle: &Entity<Workspace>,
) -> impl Fn(&ClickEvent, &mut Window, &mut App) + 'static {
    let handle = handle.clone();
    move |_, _, cx| {
        handle.update(cx, |workspace, cx| {
            workspace.set_page(Page::Diagnostics, cx)
        })
    }
}
