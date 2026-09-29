//! The Docker daemon settings of [`HostModel`]. See feature 0020.

use captain_core::daemon::DaemonSettings;
use gpui_kit::*;

use super::HostModel;
use crate::settings;

impl HostModel {
    /// The saved Docker daemon settings.
    pub fn daemon(&self, cx: &App) -> DaemonSettings {
        settings::current(cx).engine_daemon
    }

    /// Saves new daemon settings. The next start of the engine applies them.
    pub fn set_daemon(&mut self, daemon: DaemonSettings, cx: &mut Context<Self>) {
        self.host.set_daemon(daemon.clone());
        settings::update(cx, |settings| settings.engine_daemon = daemon);
        cx.notify();
    }

    /// True if the engine runs with daemon settings other than the saved ones, so a
    /// restart would apply them.
    pub fn daemon_needs_restart(&self, cx: &App) -> bool {
        self.status.is_running()
            && self
                .host
                .running_daemon()
                .is_some_and(|running| running != self.daemon(cx).state())
    }
}
