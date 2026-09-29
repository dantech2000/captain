use std::time::Duration;

use super::{MigrationRun, RunSummary};
use crate::migration::{
    MigrationItem, MigrationPlan, RollbackStatus, StepStatus, SubStatus, SwitchOverStep,
};

fn volume(name: &str) -> MigrationItem {
    MigrationItem::Volume {
        name: name.into(),
        size: Some(10),
        used_by_running: Vec::new(),
    }
}

fn plan() -> MigrationPlan {
    let mut plan = MigrationPlan::new(
        "unix:///old.sock",
        vec![volume("a"), volume("b"), volume("c")],
    );
    plan.toggle("volume:c");
    plan
}

#[test]
fn runs_selected_items_in_order() {
    let mut run = MigrationRun::new(&plan());
    assert_eq!(run.entries.len(), 2);
    assert_eq!(run.next_pending(), Some(0));
    run.set_status(0, StepStatus::Running { done: 5, total: 10 });
    assert!(run.is_running());
    assert_eq!(run.fraction(), 0.25);
    assert_eq!(run.next_pending(), Some(1));
    run.set_status(0, StepStatus::Done);
    run.set_status(1, StepStatus::Failed("boom".into()));
    assert_eq!(run.next_pending(), None);
    assert_eq!(
        run.summary(),
        RunSummary {
            copied: 1,
            failed: 1,
            ..RunSummary::default()
        }
    );
}

#[test]
fn retry_requeues_only_failed_items() {
    let mut run = MigrationRun::new(&plan());
    run.set_status(0, StepStatus::Done);
    run.set_status(1, StepStatus::Failed("boom".into()));
    run.set_note(1, "note".into());
    assert!(!run.retry(0));
    assert!(run.retry(1));
    assert_eq!(run.next_pending(), Some(1));
    assert_eq!(run.entries[1].note, None);
}

#[test]
fn stop_requeues_the_running_item() {
    let mut run = MigrationRun::new(&plan());
    run.set_status(0, StepStatus::Running { done: 1, total: 10 });
    run.stop();
    assert_eq!(run.entries[0].status, StepStatus::Pending);
    assert_eq!(run.summary().pending, 2);
}

fn switch_plan() -> MigrationPlan {
    let web = MigrationItem::Container {
        id: "web".into(),
        name: "web".into(),
        image: "nginx".into(),
        running: true,
        size_rw: 0,
        volumes: vec!["a".into()],
    };
    let mut plan = MigrationPlan::new("unix:///old.sock", vec![volume("a"), web]);
    plan.set_switch_over("container:web", true);
    plan
}

#[test]
fn switch_over_items_track_their_steps() {
    let mut run = MigrationRun::new(&switch_plan());
    assert!(run.entries[0].switch_over.is_none());
    assert!(run.entries[1].switch_over.is_some());
    run.set_status(1, StepStatus::Running { done: 0, total: 0 });
    assert!(run.is_switching());
    run.enter_switch_step(1, SwitchOverStep::StopSource);
    run.enter_switch_step(1, SwitchOverStep::ResyncVolumes);
    run.set_status(1, StepStatus::Failed("boom".into()));
    let progress = run.entries[1].switch_over.as_ref().expect("progress");
    assert_eq!(
        progress.status(SwitchOverStep::ResyncVolumes),
        SubStatus::Failed
    );
    assert_eq!(run.switched().count(), 1);
    assert!(!run.is_switching());

    assert!(run.retry(1));
    assert_eq!(run.switched().count(), 0);
    run.enter_switch_step(1, SwitchOverStep::Done);
    run.set_downtime(1, Duration::from_secs(4));
    run.set_status(1, StepStatus::Done);
    run.set_note(1, "Runs here now.".into());
    assert_eq!(run.summary().switched, 1);
    run.set_rollback(1, RollbackStatus::Done);
    assert_eq!(run.summary().switched, 0);
    assert_eq!(run.entries[1].note, None);
    let progress = run.entries[1].switch_over.as_ref().expect("progress");
    assert_eq!(progress.downtime, Some(Duration::from_secs(4)));
}

#[test]
fn switch_over_progress_survives_a_resume() {
    let mut first = MigrationRun::new(&switch_plan());
    first.enter_switch_step(1, SwitchOverStep::Done);
    first.set_status(1, StepStatus::Done);
    let mut second = MigrationRun::new(&switch_plan());
    second.resume_from(&first);
    assert_eq!(second.switched().count(), 1);
}

#[test]
fn resume_skips_items_already_done() {
    let mut first = MigrationRun::new(&plan());
    first.set_status(0, StepStatus::Done);
    first.set_status(1, StepStatus::Failed("boom".into()));
    let mut second = MigrationRun::new(&plan());
    second.resume_from(&first);
    assert_eq!(second.entries[0].status, StepStatus::Done);
    assert_eq!(second.entries[1].status, StepStatus::Pending);
    assert_eq!(second.next_pending(), Some(1));
}
