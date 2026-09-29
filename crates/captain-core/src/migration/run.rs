use super::{MigrationItem, MigrationPlan, StepStatus};

/// One selected item in a run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunEntry {
    pub item: MigrationItem,
    pub snapshot: bool,
    pub status: StepStatus,
    /// Something the user should know about a finished item, for example that a
    /// project was recreated container by container.
    pub note: Option<String>,
}

/// The counts for the summary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RunSummary {
    pub copied: usize,
    pub skipped: usize,
    pub failed: usize,
    /// Items that did not run, because the user stopped the run.
    pub pending: usize,
}

/// The selected items of a plan in step order, and how each went. Items run one at
/// a time. A stopped run resumes with the items that are not done.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MigrationRun {
    pub entries: Vec<RunEntry>,
}

impl MigrationRun {
    pub fn new(plan: &MigrationPlan) -> Self {
        let entries = plan
            .selected()
            .map(|entry| RunEntry {
                item: entry.item.clone(),
                snapshot: entry.snapshot,
                status: StepStatus::Pending,
                note: None,
            })
            .collect();
        Self { entries }
    }

    /// Keeps the result of items that `previous` already copied or skipped, so a new
    /// run of the same plan does not copy them again.
    pub fn resume_from(&mut self, previous: &MigrationRun) {
        for entry in &mut self.entries {
            let key = entry.item.key();
            if let Some(old) = previous.entries.iter().find(|old| old.item.key() == key)
                && old.status.is_settled()
            {
                entry.status = old.status.clone();
                entry.note = old.note.clone();
            }
        }
    }

    /// The next item to copy.
    pub fn next_pending(&self) -> Option<usize> {
        self.entries
            .iter()
            .position(|e| e.status == StepStatus::Pending)
    }

    pub fn set_status(&mut self, ix: usize, status: StepStatus) {
        if let Some(entry) = self.entries.get_mut(ix) {
            entry.status = status;
        }
    }

    pub fn set_note(&mut self, ix: usize, note: String) {
        if let Some(entry) = self.entries.get_mut(ix) {
            entry.note = Some(note);
        }
    }

    /// Puts a failed item back in the queue. Returns false if it had not failed.
    pub fn retry(&mut self, ix: usize) -> bool {
        match self.entries.get_mut(ix) {
            Some(entry) if matches!(entry.status, StepStatus::Failed(_)) => {
                entry.status = StepStatus::Pending;
                entry.note = None;
                true
            }
            _ => false,
        }
    }

    /// Puts the running item back in the queue, after the user stopped the run.
    pub fn stop(&mut self) {
        for entry in &mut self.entries {
            if matches!(entry.status, StepStatus::Running { .. }) {
                entry.status = StepStatus::Pending;
            }
        }
    }

    pub fn is_running(&self) -> bool {
        self.entries
            .iter()
            .any(|e| matches!(e.status, StepStatus::Running { .. }))
    }

    pub fn summary(&self) -> RunSummary {
        let mut summary = RunSummary::default();
        for entry in &self.entries {
            match entry.status {
                StepStatus::Done => summary.copied += 1,
                StepStatus::Skipped(_) => summary.skipped += 1,
                StepStatus::Failed(_) => summary.failed += 1,
                StepStatus::Pending | StepStatus::Running { .. } => summary.pending += 1,
            }
        }
        summary
    }

    /// The share of items that are finished, from 0.0 to 1.0.
    pub fn fraction(&self) -> f32 {
        if self.entries.is_empty() {
            return 1.0;
        }
        let done: f32 = self.entries.iter().map(|e| e.status.fraction()).sum();
        done / self.entries.len() as f32
    }
}

#[cfg(test)]
mod tests;
