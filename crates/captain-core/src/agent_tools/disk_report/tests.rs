use super::disk_report;
use crate::model::DiskUsage;

#[test]
fn lists_categories_and_cleanup_groups_for_any_engine() {
    let usage = DiskUsage {
        images_bytes: 3 << 30,
        volumes_bytes: 1 << 30,
        ..DiskUsage::default()
    };
    let report = disk_report(&usage, None, None, 1_700_000_000);
    assert_eq!(report.used, "4.0 GB");
    assert_eq!(report.capacity, None);
    let names: Vec<&str> = report.categories.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(names, ["Images", "Volumes", "Build cache", "Containers"]);
    assert_eq!(report.cleanup.len(), 5);
    assert!(report.text().starts_with("4.0 GB used."));
}
