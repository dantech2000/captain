use super::*;
use crate::GIB;
use crate::model::{Image, Volume};
use crate::storage::{ReclaimGroup, ReclaimPlan};

#[test]
fn totals_add_up_across_the_bar_and_the_checked_groups() {
    let usage = DiskUsage {
        images: vec![Image {
            id: "sha256:a".into(),
            size: 2 * GIB,
            dangling: true,
            ..Image::default()
        }],
        volumes: vec![Volume {
            name: "data".into(),
            size_bytes: Some(GIB),
            containers: Some(0),
            ..Volume::default()
        }],
        images_bytes: 8 * GIB,
        volumes_bytes: 4 * GIB,
        build_cache_bytes: 3 * GIB,
        containers_bytes: GIB,
        ..DiskUsage::default()
    };

    let captain = DiskBreakdown::new(&usage, Some(2 * GIB), Some(64 * GIB));
    assert_eq!(captain.used, 18 * GIB);
    let images = captain.categories[0];
    assert_eq!(images.category, Category::Images);
    assert!((images.fraction - 0.125).abs() < 1e-6);

    // Another engine has no snapshots and no known size, so the bar is full.
    let external = DiskBreakdown::new(&usage, None, None);
    assert_eq!(external.used, 16 * GIB);
    assert_eq!(external.categories.len(), 4);
    let sum: f32 = external.categories.iter().map(|c| c.fraction).sum();
    assert!((sum - 1.0).abs() < 1e-6);

    let plan = ReclaimPlan::new(&usage, 0);
    let defaults: Vec<_> = ReclaimGroup::ALL
        .into_iter()
        .filter(|g| g.checked_by_default())
        .collect();
    assert_eq!(plan.total_bytes(&defaults), 2 * GIB);
    assert_eq!(plan.total_bytes(&ReclaimGroup::ALL), 3 * GIB);
}
