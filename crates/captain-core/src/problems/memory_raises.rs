//! The memory limits Captain raised after an out-of-memory kill. `inspect` keeps
//! OOMKilled until the next run, but shows the new limit at once, so without this
//! the fix would offer a second raise for a run that never used the new limit.

use std::collections::HashMap;

use crate::model::ContainerDetail;

#[derive(Debug, Default)]
pub struct MemoryRaises {
    /// Each raised container, and the start time of the run the kernel killed.
    runs: HashMap<String, String>,
}

impl MemoryRaises {
    /// Notes that the limit of `detail`'s container went up after its last run.
    pub fn record(&mut self, detail: &ContainerDetail) {
        self.runs
            .insert(detail.id.clone(), detail.started_at.clone());
    }

    /// Forgets a raise that failed.
    pub fn forget(&mut self, id: &str) {
        self.runs.remove(id);
    }

    /// The limit the last run ran out of, when the kernel killed it at the current
    /// limit. None after a raise, until a run with the new limit starts.
    pub fn oom_limit(&self, detail: &ContainerDetail) -> Option<u64> {
        (detail.oom_killed && detail.memory_limit > 0 && !self.raised(detail))
            .then_some(detail.memory_limit)
    }

    /// True if Captain raised the limit after the run `detail` shows, which the
    /// kernel killed. The next run uses the new limit.
    pub fn raised(&self, detail: &ContainerDetail) -> bool {
        self.runs.get(&detail.id) == Some(&detail.started_at)
    }
}

#[cfg(test)]
mod tests;
