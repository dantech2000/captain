use super::MigrationItem;
use crate::migration::Step;

fn container(size_rw: u64) -> MigrationItem {
    MigrationItem::Container {
        id: "abc".into(),
        name: "web".into(),
        image: "nginx".into(),
        running: true,
        size_rw,
    }
}

#[test]
fn keys_are_unique_per_kind() {
    let volume = MigrationItem::Volume {
        name: "web".into(),
        size: None,
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
    };
    assert!(project.loses_changes());
}
