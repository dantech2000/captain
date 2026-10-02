use std::time::Duration;

use super::{RollbackStep, SwitchOverPlan, SwitchOverStep, downtime_label};
use crate::migration::MigrationItem;

fn strings(list: &[&str]) -> Vec<String> {
    list.iter().map(ToString::to_string).collect()
}

fn project(running_services: &[&str]) -> MigrationItem {
    MigrationItem::ComposeProject {
        name: "stokecrm".into(),
        working_dir: Some("/src/stokecrm".into()),
        config_files: strings(&["docker-compose.yml"]),
        files_exist: true,
        containers: strings(&["stokecrm-postgres", "stokecrm-app"]),
        changed: Vec::new(),
        running_containers: running_services
            .iter()
            .map(|service| format!("stokecrm-{service}"))
            .collect(),
        running_services: strings(running_services),
        volumes: strings(&["stokecrm-pgdata"]),
    }
}

fn container(running: bool) -> MigrationItem {
    MigrationItem::Container {
        id: "abc".into(),
        name: "web".into(),
        image: "nginx".into(),
        running,
        size_rw: 0,
        volumes: strings(&["web-data"]),
    }
}

#[test]
fn steps_run_in_order() {
    let mut step = SwitchOverStep::StopSource;
    let mut order = vec![step];
    while let Some(next) = step.next() {
        order.push(next);
        step = next;
    }
    assert_eq!(order, SwitchOverStep::SEQUENCE);
    assert_eq!(
        order,
        [
            SwitchOverStep::StopSource,
            SwitchOverStep::ResyncVolumes,
            SwitchOverStep::StartTarget,
            SwitchOverStep::Verify,
            SwitchOverStep::Done,
        ]
    );
    assert_eq!(
        RollbackStep::SEQUENCE,
        [RollbackStep::StopTarget, RollbackStep::StartSource]
    );
}

#[test]
fn a_plan_stops_and_restarts_only_what_ran_and_resyncs_its_volumes() {
    // app, backup, mail, and offsite sit behind profiles and were not running.
    let plan = SwitchOverPlan::for_item(&project(&["postgres"])).expect("plan");
    assert_eq!(plan.services, ["postgres"]);
    assert_eq!(plan.stop, ["stokecrm-postgres"]);
    assert_eq!(plan.volumes, ["stokecrm-pgdata"]);
    let plan = SwitchOverPlan::for_item(&container(true)).expect("plan");
    assert_eq!(plan.stop, ["web"]);
    assert_eq!(plan.volumes, ["web-data"]);
    assert!(plan.services.is_empty());
}

#[test]
fn only_running_items_switch_over() {
    assert!(SwitchOverPlan::for_item(&container(false)).is_none());
    assert!(SwitchOverPlan::for_item(&project(&[])).is_none());
    let volume = MigrationItem::Volume {
        name: "data".into(),
        size: None,
        used_by_running: strings(&["db"]),
    };
    assert!(SwitchOverPlan::for_item(&volume).is_none());
}

#[test]
fn downtime_reads_in_seconds_and_minutes() {
    assert_eq!(downtime_label(Duration::from_millis(200)), "under 1 s");
    assert_eq!(downtime_label(Duration::from_millis(4200)), "4 s");
    assert_eq!(downtime_label(Duration::from_secs(60)), "1 min");
    assert_eq!(downtime_label(Duration::from_secs(65)), "1 min 5 s");
}
