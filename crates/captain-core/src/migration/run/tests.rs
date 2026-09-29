use super::{MigrationRun, RunSummary};
use crate::migration::{MigrationItem, MigrationPlan, StepStatus};

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
