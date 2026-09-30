use captain_core::storage::{ReclaimGroup, ReclaimItem, ReclaimTarget};

use super::note;

fn item(group: ReclaimGroup) -> ReclaimItem {
    ReclaimItem {
        group,
        name: "node:22-alpine".into(),
        size: 1,
        time: 0,
        target: ReclaimTarget::Volume {
            name: "data".into(),
        },
    }
}

#[test]
fn warns_about_volumes_only_when_the_list_has_one() {
    let images = [item(ReclaimGroup::UnusedImages)];
    assert_eq!(note(&images, false), "These items are removed.");
    let volumes = [
        item(ReclaimGroup::UnusedImages),
        item(ReclaimGroup::UnusedVolumes),
    ];
    assert!(note(&volumes, false).contains("Volumes cannot come back"));
}
