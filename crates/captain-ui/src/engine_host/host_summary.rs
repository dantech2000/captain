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
        let text = match self.status {
            HostStatus::Running => "Engine running",
            HostStatus::Starting => "Engine starting...",
            HostStatus::Stopping => "Engine stopping...",
            HostStatus::NotCreated => "Engine not set up",
            HostStatus::Stopped => "Engine stopped",
            HostStatus::Failed(_) => "Engine failed",
            HostStatus::NotInstalled(_) => "Lima not installed",
        };
        (palette.host_status(&self.status), text)
    }
}
