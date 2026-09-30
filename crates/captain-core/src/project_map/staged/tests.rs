use super::{Setting, StagedChange, StagedChanges};
use crate::model::{ResourceUpdate, RestartPolicy};

const MIB: u64 = 1024 * 1024;

fn change(id: &str, from: Setting, to: Setting) -> StagedChange {
    StagedChange {
        container_id: id.into(),
        container: format!("svc-{id}"),
        from,
        to,
    }
}

#[test]
fn staging_the_same_field_replaces_and_going_back_removes() {
    let mut staged = StagedChanges::default();
    staged.stage(change(
        "w",
        Setting::Memory(256 * MIB),
        Setting::Memory(512 * MIB),
    ));
    // The second edit starts from the staged value; the row keeps the real old one.
    staged.stage(change(
        "w",
        Setting::Memory(512 * MIB),
        Setting::Memory(MIB << 10),
    ));
    assert_eq!(staged.changes().len(), 1);
    assert_eq!(staged.changes()[0].from, Setting::Memory(256 * MIB));
    assert_eq!(staged.changes()[0].to, Setting::Memory(MIB << 10));
    // A change to the same value stages nothing.
    staged.stage(change("w", Setting::Cpus(0), Setting::Cpus(0)));
    assert_eq!(staged.changes().len(), 1);
    staged.stage(change(
        "w",
        Setting::Memory(MIB << 10),
        Setting::Memory(256 * MIB),
    ));
    assert!(staged.is_empty());
}

#[test]
fn removes_one_row_and_drops_gone_containers() {
    let mut staged = StagedChanges::default();
    staged.stage(change("a", Setting::Memory(0), Setting::Memory(512 * MIB)));
    staged.stage(change("a", Setting::Cpus(0), Setting::Cpus(2_000_000_000)));
    staged.stage(change("b", Setting::Cpus(0), Setting::Cpus(500_000_000)));
    staged.remove("a", &Setting::Cpus(0));
    assert_eq!(staged.staged("a", &Setting::Cpus(0)), None);
    assert_eq!(
        staged.staged("a", &Setting::Memory(0)),
        Some(Setting::Memory(512 * MIB))
    );
    staged.retain_containers(|id| id != "b");
    assert!(!staged.has("b"));
    assert_eq!(staged.changes().len(), 1);
}

#[test]
fn plans_one_update_per_container_in_staging_order() {
    let mut staged = StagedChanges::default();
    staged.stage(change(
        "w",
        Setting::Memory(256 * MIB),
        Setting::Memory(512 * MIB),
    ));
    staged.stage(change(
        "a",
        Setting::Restart(RestartPolicy::No),
        Setting::Restart(RestartPolicy::Always),
    ));
    staged.stage(change("w", Setting::Cpus(0), Setting::Cpus(1_500_000_000)));
    let plan = staged.plan();
    assert_eq!(plan.len(), 2);
    assert_eq!(plan[0].container_id, "w");
    assert_eq!(
        plan[0].update,
        ResourceUpdate {
            memory: Some(512 * MIB),
            nano_cpus: Some(1_500_000_000),
            restart_policy: None,
        }
    );
    assert_eq!(plan[1].update.restart_policy, Some(RestartPolicy::Always));
    assert_eq!(Setting::Cpus(1_500_000_000).value_label(), "1.5");
}
