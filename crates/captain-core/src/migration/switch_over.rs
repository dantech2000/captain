use std::time::Duration;

use super::MigrationItem;

/// One stage of a switch-over, in the order they run. See
/// docs/adr/0009-migration.md, "Switch-over mode".
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SwitchOverStep {
    /// Stop the item's running containers in the source. They are never removed.
    StopSource,
    /// Empty the item's volumes in the target and copy them again, now that nothing
    /// writes to them.
    ResyncVolumes,
    /// Start the item in the target: the services that ran in the source, or the
    /// recreated container.
    StartTarget,
    /// Wait for health or a steady run, then connect to the published ports.
    Verify,
    Done,
}

impl SwitchOverStep {
    pub const SEQUENCE: [SwitchOverStep; 5] = [
        SwitchOverStep::StopSource,
        SwitchOverStep::ResyncVolumes,
        SwitchOverStep::StartTarget,
        SwitchOverStep::Verify,
        SwitchOverStep::Done,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::StopSource => "Stop in the old engine",
            Self::ResyncVolumes => "Copy data again",
            Self::StartTarget => "Start here",
            Self::Verify => "Check",
            Self::Done => "Done",
        }
    }

    /// The step after this one, or `None` after [`Self::Done`].
    pub fn next(self) -> Option<Self> {
        let ix = Self::SEQUENCE.iter().position(|step| *step == self)?;
        Self::SEQUENCE.get(ix + 1).copied()
    }
}

/// One stage of a roll back, in the order they run. A roll back undoes a
/// switch-over: the copy stops, and the original runs again.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RollbackStep {
    /// Stop the item's containers in the target. They are not removed either.
    StopTarget,
    /// Start the containers that the switch-over stopped in the source.
    StartSource,
}

impl RollbackStep {
    pub const SEQUENCE: [RollbackStep; 2] = [RollbackStep::StopTarget, RollbackStep::StartSource];

    pub fn label(self) -> &'static str {
        match self {
            Self::StopTarget => "Stop here",
            Self::StartSource => "Start in the old engine",
        }
    }
}

/// What a switch-over of one item touches. Built from the scanned item, so it names
/// only what ran when the source was read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwitchOverPlan {
    /// Containers to stop in the source, by name. A roll back starts them again.
    pub stop: Vec<String>,
    /// Volumes to copy again after the stop: the item's own named volumes.
    pub volumes: Vec<String>,
    /// Compose services to start in the target: those that ran in the source. It is
    /// empty for a standalone container.
    pub services: Vec<String>,
}

impl SwitchOverPlan {
    /// The plan for `item`, or `None` if it cannot switch over.
    pub fn for_item(item: &MigrationItem) -> Option<Self> {
        if !item.can_switch_over() {
            return None;
        }
        match item {
            MigrationItem::Container { name, volumes, .. } => Some(Self {
                stop: vec![name.clone()],
                volumes: volumes.clone(),
                services: Vec::new(),
            }),
            MigrationItem::ComposeProject {
                running_services,
                running_containers,
                volumes,
                ..
            } => Some(Self {
                stop: running_containers.clone(),
                volumes: volumes.clone(),
                services: running_services.clone(),
            }),
            _ => None,
        }
    }
}

/// A short duration for the downtime, for example `4 s` or `1 min 5 s`.
pub fn downtime_label(downtime: Duration) -> String {
    let seconds = downtime.as_secs_f64().round() as u64;
    match (seconds / 60, seconds % 60) {
        (0, 0) => "under 1 s".into(),
        (0, s) => format!("{s} s"),
        (m, 0) => format!("{m} min"),
        (m, s) => format!("{m} min {s} s"),
    }
}

#[cfg(test)]
mod tests;
