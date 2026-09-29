use super::VolumeStore;
use crate::model::Volume;
use crate::store::UsageFilter;

fn volume(name: &str, project: Option<&str>, size: Option<u64>, used: Option<usize>) -> Volume {
    Volume {
        name: name.into(),
        compose_project: project.map(Into::into),
        size_bytes: size,
        containers: used,
        ..Volume::default()
    }
}

fn store() -> VolumeStore {
    let mut store = VolumeStore::default();
    store.replace(vec![
        volume(&"f".repeat(64), None, Some(10), Some(0)),
        volume("pgdata", Some("shop"), Some(1024 * 1024 * 1024), Some(1)),
        volume("Cache", None, Some(400 * 1024 * 1024), Some(0)),
        volume("assets", Some("shop"), None, None),
    ]);
    store
}

#[test]
fn sorts_by_name_with_anonymous_last() {
    let names: Vec<_> = store()
        .volumes()
        .iter()
        .map(|v| v.display_name().to_string())
        .collect();
    assert_eq!(names, ["assets", "Cache", "pgdata", "ffffffffffff"]);
}

#[test]
fn groups_by_project_after_filtering() {
    let store = store();
    let groups = store.groups(UsageFilter::All);
    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0].project.as_deref(), Some("shop"));
    assert_eq!(groups[0].items.len(), 2);
    assert_eq!(groups[1].project, None);

    let unused = store.groups(UsageFilter::Unused);
    assert_eq!(unused.len(), 1);
    assert_eq!(unused[0].items.len(), 2);
    assert_eq!(store.count(UsageFilter::InUse), 1);
}

#[test]
fn summary_adds_known_sizes() {
    assert_eq!(store().summary(), "4 volumes · 1.4 GB");

    let mut unknown = VolumeStore::default();
    unknown.replace(vec![volume("a1", None, None, None)]);
    assert_eq!(unknown.total_size(), None);
    assert_eq!(unknown.summary(), "1 volume");
}

#[test]
fn first_follows_display_order() {
    let store = store();
    assert_eq!(store.first(UsageFilter::All).unwrap().name, "assets");
    assert_eq!(store.first(UsageFilter::Unused).unwrap().name, "Cache");
    assert!(VolumeStore::default().first(UsageFilter::All).is_none());
}
