use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::model::{EngineEvent, EventKind};

/// A container that crashed this recently still counts as crashing, even while it
/// runs between restarts. A crash loop spends most of its time running, so the
/// `Restarting` state alone shows it only now and then.
pub const RECENT: Duration = Duration::from_secs(60);

/// What the event stream said about a container's recent exits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Crash {
    /// When it last exited on its own.
    pub at: Instant,
    /// The kernel killed it for memory (an `oom` event) since its last clean start.
    pub out_of_memory: bool,
}

/// Remembers containers that exit on their own, from the engine's events. A `docker
/// stop` or `docker kill` sends `kill` before `die` and is not a crash; it also
/// clears the container's last crash, so stopping a crash loop ends its problem. Docker clears
/// a container's `OOMKilled` flag at every start, but its `oom` event stays here.
/// See <https://docs.docker.com/reference/cli/docker/system/events/>.
#[derive(Debug, Default)]
pub struct CrashTracker {
    crashes: HashMap<String, Crash>,
    killed: HashMap<String, bool>,
    out_of_memory: HashMap<String, bool>,
}

impl CrashTracker {
    pub fn record(&mut self, event: &EngineEvent) {
        self.record_at(event, Instant::now());
    }

    /// Records `event` as received at `at`.
    pub fn record_at(&mut self, event: &EngineEvent, at: Instant) {
        if event.kind != EventKind::Container {
            return;
        }
        let id = event.id.clone();
        match event.action.as_str() {
            "kill" => {
                self.crashes.remove(&id);
                self.killed.insert(id, true);
            }
            // A stop during a restart delay sends no `kill`: nothing runs.
            "stop" => {
                self.crashes.remove(&id);
            }
            "oom" => {
                self.out_of_memory.insert(id, true);
            }
            "die" if self.killed.remove(&id).unwrap_or(false) => {
                self.out_of_memory.remove(&id);
            }
            "die" => {
                let out_of_memory = self.out_of_memory.remove(&id).unwrap_or(false);
                self.crashes.insert(id, Crash { at, out_of_memory });
            }
            "destroy" => {
                self.crashes.remove(&id);
                self.killed.remove(&id);
                self.out_of_memory.remove(&id);
            }
            _ => {}
        }
    }

    /// The last crash of `id`, if it was within [`RECENT`] of `now`.
    pub fn recent(&self, id: &str, now: Instant) -> Option<Crash> {
        self.crashes
            .get(id)
            .copied()
            .filter(|crash| now.saturating_duration_since(crash.at) < RECENT)
    }

    /// How long after `now` the next recent crash stops counting, if any. Views
    /// read crashes only when they draw, so the app redraws at that moment.
    pub fn next_expiry(&self, now: Instant) -> Option<Duration> {
        self.crashes
            .values()
            .map(|crash| RECENT.saturating_sub(now.saturating_duration_since(crash.at)))
            .filter(|left| !left.is_zero())
            .min()
    }
}

#[cfg(test)]
mod tests;
