use captain_core::HostStatus;

use super::segments::{Segment, engine_name};
use crate::engine_host::HostSummary;
use crate::theme::Palette;
use crate::workspace::{Connection, EngineHealth, Page, Workspace};

/// The end of the engine segment's sentence: a click opens Diagnostics, whose Engine
/// card has Start, Stop, and Restart.
const CONTROLS: &str = "Click to start, stop, or restart it on the Diagnostics page.";
const DIAGNOSTICS: &str = "Click to open Diagnostics.";
/// Added while Captain reconnects by itself. See feature 0013.
const RETRYING: &str = "Captain keeps trying to reconnect.";

/// The engine segment. With Captain Engine the state comes from the host, except
/// that a running engine whose Docker does not answer shows "reconnecting" in the
/// warning color, and then the failure in red.
pub fn engine(workspace: &Workspace, host: Option<&HostSummary>, palette: &Palette) -> Segment {
    let connection = workspace.connection();
    let health = workspace.engine_health(host.map(|host| &host.status));
    let retry = if workspace.reconnecting().is_some() {
        format!(" {RETRYING}")
    } else {
        String::new()
    };
    let error = match connection {
        Connection::Failed(error) => error.to_string(),
        _ => String::new(),
    };
    if let Some(host) = host {
        let (dot, color, help) = match health {
            EngineHealth::Reconnecting => (
                palette.orange,
                Some(palette.warn_text),
                format!(
                    "Captain Engine runs, but Docker stopped answering, for example after a \
                     restart. Captain is reconnecting. {CONTROLS}"
                ),
            ),
            EngineHealth::NotAnswering => (
                palette.red,
                Some(palette.red),
                format!(
                    "Captain Engine runs, but Docker does not answer: {error}.{retry} {CONTROLS}"
                ),
            ),
            EngineHealth::Running | EngineHealth::Connecting => (
                palette.green,
                None,
                format!("Captain Engine is running. {CONTROLS}"),
            ),
            EngineHealth::Starting => (
                palette.orange,
                None,
                format!(
                    "Captain Engine is {}. {CONTROLS}",
                    host.status.label().to_lowercase()
                ),
            ),
            EngineHealth::NotSetUp => (
                palette.gray,
                None,
                "Captain Engine is not set up. Click to set it up on the Diagnostics page."
                    .to_string(),
            ),
            EngineHealth::CannotRun => {
                let why = match &host.status {
                    HostStatus::Failed(why) | HostStatus::NotInstalled(why) => why.as_str(),
                    _ => "",
                };
                (
                    palette.red,
                    None,
                    format!("Captain Engine cannot run: {why} {DIAGNOSTICS}"),
                )
            }
            EngineHealth::Stopped => (
                palette.red,
                None,
                format!("Captain Engine is stopped. {CONTROLS}"),
            ),
        };
        return Segment {
            id: "status-engine",
            label: "Captain Engine".into(),
            help: help.into(),
            dot: Some(dot),
            color,
            page: Some(Page::Diagnostics),
        };
    }
    let (label, dot, color, help) = match (health, connection) {
        (_, Connection::Connected(info)) => {
            let name = engine_name(&info.endpoint);
            let help = format!("Captain uses {name} at {}. {DIAGNOSTICS}", info.endpoint);
            (name.to_string(), palette.green, None, help)
        }
        (EngineHealth::Reconnecting, _) => (
            "Engine reconnecting".to_string(),
            palette.orange,
            Some(palette.warn_text),
            format!("The engine stopped answering. Captain is reconnecting. {DIAGNOSTICS}"),
        ),
        (EngineHealth::NotAnswering, _) => (
            "Engine".to_string(),
            palette.red,
            Some(palette.red),
            format!("Captain cannot reach the engine: {error}.{retry} {DIAGNOSTICS}"),
        ),
        _ => (
            "Engine".to_string(),
            palette.orange,
            None,
            format!("Captain is connecting to the engine. {DIAGNOSTICS}"),
        ),
    };
    Segment {
        id: "status-engine",
        label: label.into(),
        help: help.into(),
        dot: Some(dot),
        color,
        page: Some(Page::Diagnostics),
    }
}
