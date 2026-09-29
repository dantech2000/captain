use captain_core::snapshot::SnapshotMetadata;
use captain_core::{GIB, HostResources};

use super::{finish, new_id, read_all, remove_incomplete};

fn metadata(name: &str, created: u64) -> SnapshotMetadata {
    SnapshotMetadata {
        name: name.into(),
        description: String::new(),
        created,
        captain_version: String::new(),
        lima_version: String::new(),
        disk_allocated: 0,
        resources: HostResources {
            cpus: 2,
            memory_bytes: 4 * GIB,
            disk_bytes: 16 * GIB,
        },
    }
}

#[test]
fn lists_only_complete_snapshots_and_cleans_up_the_rest() {
    let root = std::env::temp_dir().join(format!("captain-snapfolder-{}", std::process::id()));
    std::fs::remove_dir_all(&root).ok();
    for (id, name, created) in [("a", "first", 1), ("b", "second", 2)] {
        std::fs::create_dir_all(root.join(id)).unwrap();
        finish(&root.join(id), &metadata(name, created)).unwrap();
    }
    std::fs::create_dir_all(root.join("half-made")).unwrap();

    let names: Vec<_> = read_all(&root)
        .into_iter()
        .map(|s| s.metadata.name)
        .collect();
    assert_eq!(names, ["second", "first"]);
    remove_incomplete(&root);
    assert!(!root.join("half-made").exists());
    assert!(root.join("a").exists());
    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn ids_are_distinct_version_4_uuids() {
    let (a, b) = (new_id(), new_id());
    assert_ne!(a, b);
    assert_eq!(a.len(), 36);
    assert_eq!(a.as_bytes()[14], b'4');
}
