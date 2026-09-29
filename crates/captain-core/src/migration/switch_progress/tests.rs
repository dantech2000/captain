use super::{RollbackStatus, SubStatus, SwitchOverProgress};
use crate::migration::SwitchOverStep;

#[test]
fn statuses_follow_the_current_step() {
    let mut progress = SwitchOverProgress::default();
    assert_eq!(
        progress.status(SwitchOverStep::StopSource),
        SubStatus::Pending
    );
    assert!(!progress.touched_source());
    progress.enter(SwitchOverStep::ResyncVolumes);
    assert_eq!(progress.status(SwitchOverStep::StopSource), SubStatus::Done);
    assert_eq!(
        progress.status(SwitchOverStep::ResyncVolumes),
        SubStatus::Running
    );
    assert_eq!(progress.status(SwitchOverStep::Verify), SubStatus::Pending);
    progress.enter(SwitchOverStep::Done);
    assert!(progress.is_done());
    assert_eq!(progress.status(SwitchOverStep::Done), SubStatus::Done);
}

#[test]
fn a_failure_marks_the_current_step() {
    let mut progress = SwitchOverProgress::default();
    progress.fail();
    assert_eq!(
        progress.status(SwitchOverStep::StopSource),
        SubStatus::Pending
    );
    progress.enter(SwitchOverStep::Verify);
    progress.fail();
    assert_eq!(progress.status(SwitchOverStep::Verify), SubStatus::Failed);
    assert_eq!(
        progress.status(SwitchOverStep::StartTarget),
        SubStatus::Done
    );
    assert!(progress.can_roll_back());
}

#[test]
fn roll_back_is_offered_once_the_source_was_reached() {
    let mut progress = SwitchOverProgress::default();
    assert!(!progress.can_roll_back());
    progress.enter(SwitchOverStep::Done);
    assert!(progress.can_roll_back());
    progress.rollback = RollbackStatus::Running;
    assert!(!progress.can_roll_back());
    progress.rollback = RollbackStatus::Failed("boom".into());
    assert!(progress.can_roll_back());
    progress.rollback = RollbackStatus::Done;
    assert!(!progress.can_roll_back());
}
