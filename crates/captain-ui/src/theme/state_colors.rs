//! The one color for each state of a container, health check, engine, and
//! diagnostics check.

use captain_core::HostStatus;
use captain_core::diagnostics::CheckState;
use captain_core::model::{ContainerState, Health};
use gpui_kit::*;

use super::Palette;

impl Palette {
    pub fn container_state(&self, state: ContainerState) -> Hsla {
        match state {
            ContainerState::Running => self.green,
            ContainerState::Paused | ContainerState::Restarting => self.orange,
            ContainerState::Dead => self.red,
            _ => self.gray,
        }
    }

    pub fn health(&self, health: Health) -> Hsla {
        match health {
            Health::Healthy => self.green,
            Health::Starting => self.orange,
            Health::Unhealthy => self.red,
        }
    }

    pub fn host_status(&self, status: &HostStatus) -> Hsla {
        match status {
            HostStatus::Running => self.green,
            HostStatus::Starting | HostStatus::Stopping => self.orange,
            HostStatus::NotCreated => self.gray,
            _ => self.red,
        }
    }

    pub fn check_state(&self, state: CheckState) -> Hsla {
        match state {
            CheckState::Passed => self.green,
            CheckState::Warning => self.orange,
            CheckState::Failed => self.red,
            CheckState::NotApplicable => self.gray,
        }
    }
}
