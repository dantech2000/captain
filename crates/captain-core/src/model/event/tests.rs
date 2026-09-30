use super::{EngineEvent, EventKind};

fn event(kind: EventKind, action: &str) -> EngineEvent {
    EngineEvent {
        kind,
        action: action.into(),
        id: "abc".into(),
        ..EngineEvent::default()
    }
}

#[test]
fn lifecycle_container_events_change_the_list() {
    for action in ["create", "start", "die", "destroy", "pause"] {
        assert!(event(EventKind::Container, action).changes_container_list());
    }
}

#[test]
fn other_events_do_not_change_the_list() {
    for action in ["exec_start: sh", "health_status: healthy", "attach", "top"] {
        assert!(!event(EventKind::Container, action).changes_container_list());
    }
    assert!(!event(EventKind::Image, "delete").changes_container_list());
    assert!(!event(EventKind::Network, "create").changes_container_list());
}

#[test]
fn only_events_that_add_or_remove_data_change_the_disk_use() {
    assert!(event(EventKind::Image, "pull").changes_disk_use());
    assert!(event(EventKind::Image, "tag").changes_disk_use());
    assert!(event(EventKind::Volume, "destroy").changes_disk_use());
    assert!(event(EventKind::Container, "create").changes_disk_use());
    assert!(event(EventKind::Other, "prune").changes_disk_use());
    assert!(!event(EventKind::Container, "start").changes_disk_use());
    assert!(!event(EventKind::Volume, "mount").changes_disk_use());
    assert!(!event(EventKind::Image, "push").changes_disk_use());
}
