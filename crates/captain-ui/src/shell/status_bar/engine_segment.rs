use captain_core::HostStatus;

use super::segments::{Segment, engine_name};
use crate::engine_host::HostSummary;
use crate::theme::Palette;
use crate::workspace::{Connection, Page, Workspace};

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
    let retrying = workspace.reconnecting().is_some();
    let retry = if retrying {
        format!(" {RETRYING}")
    } else {
        String::new()
    };
    if let Some(host) = host {
        let (dot, color, help) = match (&host.status, connection) {
            (HostStatus::Running, Connection::Connecting) if retrying => (
                palette.orange,
                Some(palette.warn_text),
                format!(
                    "Captain Engine runs, but Docker stopped answering, for example after a \
                     restart. Captain is reconnecting. {CONTROLS}"
                ),
            ),
            (HostStatus::Running, Connection::Failed(error)) => (
                palette.red,
                Some(palette.red),
                format!(
                    "Captain Engine runs, but Docker does not answer: {error}.{retry} {CONTROLS}"
                ),
            ),
            (HostStatus::Running, _) => (
                palette.green,
                None,
                format!("Captain Engine is running. {CONTROLS}"),
            ),
            (HostStatus::Starting | HostStatus::Stopping, _) => (
                palette.orange,
                None,
                format!(
                    "Captain Engine is {}. {CONTROLS}",
                    host.status.label().to_lowercase()
                ),
            ),
            (HostStatus::NotCreated, _) => (
                palette.gray,
                None,
                "Captain Engine is not set up. Click to set it up on the Diagnostics page."
                    .to_string(),
            ),
            (HostStatus::Failed(why) | HostStatus::NotInstalled(why), _) => (
                palette.red,
                None,
                format!("Captain Engine cannot run: {why} {DIAGNOSTICS}"),
            ),
            (HostStatus::Stopped, _) => (
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
    let (label, dot, color, help) = match connection {
        Connection::Connecting if retrying => (
            "Engine reconnecting".to_string(),
            palette.orange,
            Some(palette.warn_text),
            format!("The engine stopped answering. Captain is reconnecting. {DIAGNOSTICS}"),
        ),
        Connection::Connecting => (
            "Engine".to_string(),
            palette.orange,
            None,
            format!("Captain is connecting to the engine. {DIAGNOSTICS}"),
        ),
        Connection::Connected(info) => {
            let name = engine_name(&info.endpoint);
            let help = format!("Captain uses {name} at {}. {DIAGNOSTICS}", info.endpoint);
            (name.to_string(), palette.green, None, help)
        }
        Connection::Failed(error) => (
            "Engine".to_string(),
            palette.red,
            Some(palette.red),
            format!("Captain cannot reach the engine: {error}.{retry} {DIAGNOSTICS}"),
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
