use super::MigrationItem;
use crate::migration::Step;

fn container(size_rw: u64) -> MigrationItem {
    MigrationItem::Container {
        id: "abc".into(),
        name: "web".into(),
        image: "nginx".into(),
        running: true,
        size_rw,
        volumes: Vec::new(),
    }
}

#[test]
fn keys_are_unique_per_kind() {
    let volume = MigrationItem::Volume {
        name: "web".into(),
        size: None,
        used_by_running: Vec::new(),
    };
    assert_eq!(volume.key(), "volume:web");
    assert_eq!(container(0).key(), "container:abc");
    assert_ne!(volume.key(), container(0).key());
}

#[test]
fn untagged_image_shows_its_short_id() {
    let image = MigrationItem::Image {
        id: "sha256:0123456789abcdef".into(),
        tags: vec![],
        size: 10,
        in_use: false,
    };
    assert_eq!(image.label(), "0123456789ab");
    assert_eq!(image.step(), Step::Images);
    assert_eq!(image.size(), 10);
}

#[test]
fn flags_containers_with_changes() {
    assert!(container(4096).loses_changes());
    assert_eq!(container(4096).changed_bytes(), 4096);
    assert!(!container(0).loses_changes());
    let project = MigrationItem::ComposeProject {
        name: "shop".into(),
        working_dir: None,
        config_files: vec![],
        files_exist: false,
        containers: vec!["shop-db-1".into()],
        changed: vec!["shop-db-1".into()],
        running_services: Vec::new(),
        running_containers: Vec::new(),
        volumes: Vec::new(),
    };
    assert!(project.loses_changes());
}

#[test]
fn only_running_containers_and_projects_switch_over() {
    assert!(container(0).can_switch_over());
    assert_eq!(container(0).running_containers(), ["web"]);
    let stopped = MigrationItem::Container {
        id: "abc".into(),
        name: "web".into(),
        image: "nginx".into(),
        running: false,
        size_rw: 0,
        volumes: Vec::new(),
    };
    assert!(!stopped.can_switch_over());
    assert!(stopped.running_containers().is_empty());
    let project = |running: &[&str]| MigrationItem::ComposeProject {
        name: "shop".into(),
        working_dir: None,
        config_files: vec![],
        files_exist: false,
        containers: vec!["shop-db-1".into()],
        changed: vec![],
        running_services: running.iter().map(ToString::to_string).collect(),
        running_containers: running.iter().map(|s| format!("shop-{s}-1")).collect(),
        volumes: vec![],
    };
    assert!(project(&["db"]).can_switch_over());
    assert_eq!(project(&["db"]).running_containers(), ["shop-db-1"]);
    assert!(!project(&[]).can_switch_over());
    let volume = MigrationItem::Volume {
        name: "data".into(),
        size: None,
        used_by_running: vec!["db".into()],
    };
    assert!(!volume.can_switch_over());
}
