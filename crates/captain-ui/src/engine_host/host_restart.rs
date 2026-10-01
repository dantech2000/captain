//! "Restart to apply": what a restart of the running engine would change, its
//! resources or its Docker daemon settings. See feature 0037.

use gpui_kit::*;

use super::HostModel;

impl HostModel {
    /// The sentence for "Restart to apply", while the engine runs with other
    /// resources or Docker daemon settings than the saved ones.
    pub fn restart_note(&self, cx: &App) -> Option<String> {
        let resources = self.status.is_running().then(|| {
            let running = self.host.running_resources()?;
            self.resources().restart_change(&running)
        });
        let daemon = self.daemon_needs_restart(cx);
        match (resources.flatten(), daemon) {
            (Some((now, saved)), false) => Some(format!(
                "Restart to apply: Captain Engine runs with {now}; the new settings are {saved}."
            )),
            (Some((now, saved)), true) => Some(format!(
                "Restart to apply: Captain Engine runs with {now} and the previous Docker \
                 daemon settings; the new settings are {saved}."
            )),
            (None, true) => Some(
                "Restart to apply: Captain Engine runs with the previous Docker daemon settings."
                    .into(),
            ),
            (None, false) => None,
        }
    }

    /// The size of Captain Engine's disk now, or `None` before the first setup. The
    /// disk stepper does not go below it.
    pub fn current_disk(&self) -> Option<u64> {
        self.host.current_disk()
    }
}
