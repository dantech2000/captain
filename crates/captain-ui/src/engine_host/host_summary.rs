use captain_core::HostStatus;
use gpui_kit::*;

use super::{HostModel, host_model};
use crate::theme::Palette;

/// What the sidebar and the tray show about Captain Engine.
#[derive(Clone)]
pub struct HostSummary {
    pub status: HostStatus,
    pub can_control: bool,
    pub model: Entity<HostModel>,
}

/// The summary while the settings choose Captain Engine, else `None`.
pub fn summary(cx: &App) -> Option<HostSummary> {
    let model = host_model(cx)?;
    let host = model.read(cx);
    if !host.uses_captain(cx) {
        return None;
    }
    // Until the first check, show "starting" and keep the buttons off.
    let checking = host.is_checking();
    let status = if checking {
        HostStatus::Starting
    } else {
        host.status().clone()
    };
    Some(HostSummary {
        status,
        can_control: host.can_control() && !checking,
        model: model.clone(),
    })
}

impl HostSummary {
    /// The color and short text for the brand line.
    pub fn brand_line(&self, palette: &Palette) -> (Hsla, &'static str) {
        match self.status {
            HostStatus::Running => (palette.green, "Engine running"),
            HostStatus::Starting => (palette.orange, "Engine starting..."),
            HostStatus::Stopping => (palette.orange, "Engine stopping..."),
            HostStatus::NotCreated => (palette.gray, "Engine not set up"),
            HostStatus::Stopped => (palette.red, "Engine stopped"),
            HostStatus::Failed(_) => (palette.red, "Engine failed"),
            HostStatus::NotInstalled(_) => (palette.red, "Lima not installed"),
        }
    }
}
