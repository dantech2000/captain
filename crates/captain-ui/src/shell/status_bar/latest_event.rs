use std::collections::HashMap;

use chrono::{DateTime, Local};

/// The latest notable container event, for the status bar when nothing is hovered.
///
/// A container that exits on its own is notable; a `docker stop` or `docker kill` is
/// not. Docker sends `kill` before `die` for those, and only `die` for a crash or an
/// out-of-memory kill. A `start` after a crash counts as a restart, so a restart
/// policy that brings a container back shows as "restarted 3 times".
/// See <https://docs.docker.com/reference/cli/docker/system/events/>.
#[derive(Debug, Default)]
pub struct LatestEvent {
    containers: HashMap<String, Tally>,
    latest: Option<Notable>,
}

#[derive(Debug, Default)]
struct Tally {
    /// A `kill` came first, so the next `die` is on purpose.
    killed: bool,
    /// The container exited on its own and has not started since.
    crashed: bool,
    restarts: u32,
}

#[derive(Debug)]
struct Notable {
    name: String,
    restarts: u32,
    /// False after an exit, true after a restart.
    running: bool,
    at: DateTime<Local>,
}

impl LatestEvent {
    /// Records a container event. `at` is when Captain received it, because the
    /// event carries no time. True if the line changed.
    pub fn record(&mut self, action: &str, id: &str, name: &str, at: DateTime<Local>) -> bool {
        let tally = self.containers.entry(id.to_string()).or_default();
        let running = match action {
            "kill" => {
                tally.killed = true;
                return false;
            }
            "die" if std::mem::take(&mut tally.killed) => return false,
            "die" => {
                tally.crashed = true;
                false
            }
            "start" if std::mem::take(&mut tally.crashed) => {
                tally.restarts += 1;
                true
            }
            "destroy" => {
                self.containers.remove(id);
                return false;
            }
            _ => return false,
        };
        self.latest = Some(Notable {
            name: name.to_string(),
            restarts: tally.restarts,
            running,
            at,
        });
        true
    }

    /// For example "worker restarted 3 times · last at 12:07:11".
    pub fn line(&self) -> Option<String> {
        let event = self.latest.as_ref()?;
        let time = event.at.format("%H:%M:%S");
        let name = &event.name;
        Some(match (event.running, event.restarts) {
            (false, 0) => format!("{name} exited at {time}"),
            // A restart policy may bring it back again; the event cannot tell.
            (false, n) => format!("{name} exited again at {time}, after {n} restarts"),
            (true, 1) => format!("{name} restarted at {time}"),
            (true, n) => format!("{name} restarted {n} times · last at {time}"),
        })
    }
}

#[cfg(test)]
mod tests;
