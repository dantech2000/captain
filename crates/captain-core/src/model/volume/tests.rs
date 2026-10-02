use super::Volume;

fn volume(name: &str, containers: Option<usize>) -> Volume {
    Volume {
        name: name.into(),
        containers,
        ..Volume::default()
    }
}

#[test]
fn anonymous_volumes_show_a_short_name() {
    let anonymous = volume(&"a1".repeat(32), None);
    assert!(anonymous.is_anonymous());
    assert_eq!(anonymous.display_name(), "a1a1a1a1a1a1");

    let named = volume("pgdata", None);
    assert!(!named.is_anonymous());
    assert_eq!(named.display_name(), "pgdata");
}

#[test]
fn usage_follows_the_container_count() {
    let used = volume("a", Some(2));
    assert!(used.is_in_use());
    assert!(!used.is_unused());
    assert!(!used.can_remove());
    assert_eq!(used.usage_label(), "2 containers");

    let unused = volume("b", Some(0));
    assert!(!unused.is_in_use());
    assert!(unused.is_unused());
    assert!(unused.can_remove());
    assert_eq!(unused.usage_label(), "Unused");

    let unknown = volume("c", None);
    assert!(!unknown.is_in_use());
    assert!(!unknown.is_unused());
    assert!(unknown.can_remove());
    assert_eq!(unknown.usage_label(), "Unknown");
    assert_eq!(volume("d", Some(1)).usage_label(), "1 container");
}

#[test]
fn anonymous_label_or_hex_name_means_pruned_by_default() {
    let mut labelled = volume("tmp", Some(0));
    labelled
        .labels
        .insert("com.docker.volume.anonymous".into(), String::new());
    assert!(labelled.pruned_by_default());
    assert!(volume(&"b".repeat(64), Some(0)).pruned_by_default());
    assert!(!volume("pgdata", Some(0)).pruned_by_default());
}
