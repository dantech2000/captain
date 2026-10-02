use super::{ImageChoice, MigrationPlan};
use crate::migration::{MigrationItem, Step};

fn image(id: &str, size: u64, in_use: bool) -> MigrationItem {
    MigrationItem::Image {
        id: id.into(),
        tags: vec![format!("{id}:latest")],
        size,
        in_use,
    }
}

fn container(id: &str, size_rw: u64) -> MigrationItem {
    MigrationItem::Container {
        id: id.into(),
        name: id.into(),
        image: "app".into(),
        running: false,
        size_rw,
        volumes: Vec::new(),
    }
}

fn plan() -> MigrationPlan {
    MigrationPlan::new(
        "unix:///old.sock",
        vec![
            container("web", 100),
            image("unused", 1000, false),
            MigrationItem::Volume {
                name: "data".into(),
                size: Some(500),
                used_by_running: Vec::new(),
            },
            image("app", 2000, true),
            MigrationItem::Network {
                name: "backend".into(),
                driver: "bridge".into(),
            },
        ],
    )
}

#[test]
fn sorts_by_step_and_selects_all() {
    let plan = plan();
    let steps: Vec<Step> = plan.entries.iter().map(|e| e.item.step()).collect();
    assert_eq!(
        steps,
        [
            Step::Networks,
            Step::Volumes,
            Step::Images,
            Step::Images,
            Step::Containers
        ]
    );
    assert!(plan.entries.iter().all(|e| e.selected && !e.snapshot));
    assert_eq!(plan.total_bytes(), 3500);
}

#[test]
fn only_images_in_use() {
    let mut plan = plan();
    plan.set_image_choice(ImageChoice::InUse);
    assert_eq!(plan.image_choice(), ImageChoice::InUse);
    assert_eq!(plan.total_bytes(), 2500);
    plan.set_image_choice(ImageChoice::All);
    assert_eq!(plan.total_bytes(), 3500);
}

#[test]
fn toggle_clears_and_selects() {
    let mut plan = plan();
    plan.toggle("volume:data");
    assert_eq!(plan.total_bytes(), 3000);
    assert_eq!(plan.step(Step::Volumes).filter(|e| e.selected).count(), 0);
    plan.toggle("volume:data");
    assert_eq!(plan.total_bytes(), 3500);
}

#[test]
fn snapshot_adds_changes_and_clears_the_warning() {
    let mut plan = plan();
    assert_eq!(plan.lost_changes().len(), 1);
    plan.set_snapshot("container:web", true);
    assert_eq!(plan.total_bytes(), 3600);
    assert!(plan.lost_changes().is_empty());
    // Only containers take a snapshot.
    plan.set_snapshot("volume:data", true);
    assert!(
        plan.selected()
            .all(|e| e.snapshot == (e.item.key() == "container:web"))
    );
}

fn running_project() -> MigrationItem {
    MigrationItem::ComposeProject {
        name: "crm".into(),
        working_dir: None,
        config_files: vec![],
        files_exist: false,
        containers: vec!["crm-db".into()],
        changed: vec![],
        running_services: vec!["db".into()],
        running_containers: vec!["crm-db".into()],
        volumes: vec!["pgdata".into()],
    }
}

#[test]
fn switch_over_is_off_by_default_and_only_for_running_items() {
    let mut plan = MigrationPlan::new(
        "test",
        vec![
            running_project(),
            container("stopped", 0),
            MigrationItem::Volume {
                name: "pgdata".into(),
                size: None,
                used_by_running: vec!["crm-db".into()],
            },
        ],
    );
    assert!(plan.entries.iter().all(|e| !e.switch_over));
    assert!(plan.switch_overs().is_empty());
    plan.set_switch_over("volume:pgdata", true);
    plan.set_switch_over("container:stopped", true);
    assert!(plan.switch_overs().is_empty());
    plan.set_switch_over("project:crm", true);
    assert_eq!(plan.switch_overs().len(), 1);
    // A cleared item does not switch over.
    plan.toggle("project:crm");
    assert!(plan.switch_overs().is_empty());
    plan.toggle("project:crm");
    plan.set_switch_over("project:crm", false);
    assert!(plan.switch_overs().is_empty());
}

#[test]
fn live_volumes_list_running_users_until_a_switch_over() {
    let mut plan = MigrationPlan::new(
        "test",
        vec![
            running_project(),
            MigrationItem::Volume {
                name: "pgdata".into(),
                size: None,
                used_by_running: vec!["crm-db".into()],
            },
        ],
    );
    let live = plan.live_volumes();
    assert_eq!(live.len(), 1);
    assert_eq!(live[0].0, "pgdata");
    assert_eq!(live[0].1, ["crm-db".to_string()]);
    plan.set_switch_over("project:crm", true);
    assert!(plan.live_volumes().is_empty());
}

#[test]
fn a_standalone_users_live_volume_warns_and_an_unused_one_does_not() {
    let plan = MigrationPlan::new(
        "test",
        vec![
            MigrationItem::Volume {
                name: "pgdata".into(),
                size: None,
                used_by_running: vec!["db".into()],
            },
            MigrationItem::Volume {
                name: "cache".into(),
                size: None,
                used_by_running: Vec::new(),
            },
        ],
    );
    let live = plan.live_volumes();
    assert_eq!(live.len(), 1);
    assert_eq!(live[0].0, "pgdata");
    assert_eq!(live[0].1, ["db".to_string()]);
}
