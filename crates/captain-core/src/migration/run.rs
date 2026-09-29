use std::time::Duration;

use super::{
    MigrationItem, MigrationPlan, RollbackStatus, StepStatus, SwitchOverProgress, SwitchOverStep,
};

/// One selected item in a run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunEntry {
    pub item: MigrationItem,
    pub snapshot: bool,
    pub status: StepStatus,
    /// Something the user should know about a finished item, for example that a
    /// project was recreated container by container.
    pub note: Option<String>,
    /// The switch-over, when the user turned it on for this item.
    pub switch_over: Option<SwitchOverProgress>,
}

/// The counts for the summary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RunSummary {
    pub copied: usize,
    pub skipped: usize,
    pub failed: usize,
    /// Items that did not run, because the user stopped the run.
    pub pending: usize,
    /// Items that switched over and were not rolled back: they now run in the
    /// target, and their originals are stopped, not deleted, in the source.
    pub switched: usize,
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
                switch_over: entry.switch_over.then(SwitchOverProgress::default),
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
                entry.switch_over = old.switch_over.clone();
            }
        }
    }

    /// The next item to copy.
    pub fn next_pending(&self) -> Option<usize> {
        self.entries
            .iter()
            .position(|e| e.status == StepStatus::Pending)
    }

    /// Sets the status. A failure also marks the current switch-over step.
    pub fn set_status(&mut self, ix: usize, status: StepStatus) {
        if let Some(entry) = self.entries.get_mut(ix) {
            if let (StepStatus::Failed(_), Some(progress)) = (&status, &mut entry.switch_over) {
                progress.fail();
            }
            entry.status = status;
        }
    }

    /// Records that the item's switch-over entered `step`.
    pub fn enter_switch_step(&mut self, ix: usize, step: SwitchOverStep) {
        if let Some(progress) = self.progress_mut(ix) {
            progress.enter(step);
        }
    }

    pub fn set_downtime(&mut self, ix: usize, downtime: Duration) {
        if let Some(progress) = self.progress_mut(ix) {
            progress.downtime = Some(downtime);
        }
    }

    /// Records how a roll back went. A finished roll back clears the "runs here
    /// now" note, which is no longer true.
    pub fn set_rollback(&mut self, ix: usize, rollback: RollbackStatus) {
        let done = rollback == RollbackStatus::Done;
        if let Some(progress) = self.progress_mut(ix) {
            progress.rollback = rollback;
        }
        if done && let Some(entry) = self.entries.get_mut(ix) {
            entry.note = None;
        }
    }

    /// True if the item that runs now is a switch-over. The run cannot stop then,
    /// so the item is not left stopped in the source and not started in the target.
    pub fn is_switching(&self) -> bool {
        self.entries
            .iter()
            .any(|e| matches!(e.status, StepStatus::Running { .. }) && e.switch_over.is_some())
    }

    /// Items whose switch-over reached the source, so their originals may be
    /// stopped. The summary offers a roll back for each.
    pub fn switched(&self) -> impl Iterator<Item = (usize, &RunEntry)> {
        self.entries
            .iter()
            .enumerate()
            .filter(|(_, e)| e.switch_over.as_ref().is_some_and(|p| p.touched_source()))
    }

    fn progress_mut(&mut self, ix: usize) -> Option<&mut SwitchOverProgress> {
        self.entries.get_mut(ix)?.switch_over.as_mut()
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
                if let Some(progress) = &mut entry.switch_over {
                    *progress = SwitchOverProgress::default();
                }
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
        summary.switched = self
            .entries
            .iter()
            .filter_map(|e| e.switch_over.as_ref())
            .filter(|p| p.is_done() && p.rollback != RollbackStatus::Done)
            .count();
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
