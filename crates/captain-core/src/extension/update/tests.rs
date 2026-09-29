use std::cmp::Ordering;

use super::{compare_versions, update_reference};

#[test]
fn the_update_keeps_the_repository_and_changes_the_tag() {
    assert_eq!(
        update_reference("docker/disk-usage-extension:0.2.9", "").as_deref(),
        Some("docker/disk-usage-extension:latest")
    );
    assert_eq!(
        update_reference("localhost:5000/ext:1", "2.0").as_deref(),
        Some("localhost:5000/ext:2.0")
    );
    assert_eq!(update_reference("ext:1", "a/b"), None);
}

#[test]
fn versions_compare_as_numbers() {
    assert_eq!(
        compare_versions("0.10.0", "v0.9.1"),
        Some(Ordering::Greater)
    );
    assert_eq!(compare_versions("1.2", "1.2.0-rc1"), Some(Ordering::Equal));
    assert_eq!(compare_versions("0.2.9", "0.3"), Some(Ordering::Less));
    assert_eq!(compare_versions("latest", "0.3"), None);
}
