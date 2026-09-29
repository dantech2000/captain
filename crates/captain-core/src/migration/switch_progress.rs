use std::time::Duration;

use super::SwitchOverStep;

/// How one step of a switch-over stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubStatus {
    Pending,
    Running,
    Done,
    Failed,
}

/// How a roll back stands.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum RollbackStatus {
    #[default]
    NotRun,
    Running,
    Done,
    Failed(String),
}

/// How far the switch-over of one run item has got.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SwitchOverProgress {
    /// The step that runs now, or the last one that ran. `None` before the start.
    current: Option<SwitchOverStep>,
    /// True if `current` failed.
    failed: bool,
    /// From the stop in the source until the check in the target passed.
    pub downtime: Option<Duration>,
    pub rollback: RollbackStatus,
}

impl SwitchOverProgress {
    /// Records that the switch-over entered `step`.
    pub fn enter(&mut self, step: SwitchOverStep) {
        self.current = Some(step);
        self.failed = false;
    }

    /// Marks the current step as failed.
    pub fn fail(&mut self) {
        if self.current.is_some() {
            self.failed = true;
        }
    }

    pub fn status(&self, step: SwitchOverStep) -> SubStatus {
        let Some(current) = self.current else {
            return SubStatus::Pending;
        };
        match step.cmp(&current) {
            std::cmp::Ordering::Less => SubStatus::Done,
            std::cmp::Ordering::Greater => SubStatus::Pending,
            std::cmp::Ordering::Equal if self.failed => SubStatus::Failed,
            std::cmp::Ordering::Equal if step == SwitchOverStep::Done => SubStatus::Done,
            std::cmp::Ordering::Equal => SubStatus::Running,
        }
    }

    pub fn is_done(&self) -> bool {
        self.current == Some(SwitchOverStep::Done)
    }

    /// True once the switch-over reached the source, so the original may be
    /// stopped and a roll back makes sense.
    pub fn touched_source(&self) -> bool {
        self.current.is_some()
    }

    /// True if the user can roll back now: the source was reached, and no roll back
    /// runs or has finished.
    pub fn can_roll_back(&self) -> bool {
        self.touched_source()
            && matches!(
                self.rollback,
                RollbackStatus::NotRun | RollbackStatus::Failed(_)
            )
    }
}

#[cfg(test)]
mod tests;
