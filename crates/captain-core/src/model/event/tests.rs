use super::{EngineEvent, EventKind};

fn event(kind: EventKind, action: &str) -> EngineEvent {
    EngineEvent {
        kind,
        action: action.into(),
        id: "abc".into(),
    }
}

#[test]
fn lifecycle_container_events_change_the_list() {
    for action in ["create", "start", "die", "destroy", "pause"] {
        assert!(event(EventKind::Container, action).changes_container_list());
    }
}

#[test]
fn noisy_container_events_do_not_change_the_list() {
    for action in ["exec_start: sh", "health_status: healthy", "attach", "top"] {
        assert!(!event(EventKind::Container, action).changes_container_list());
    }
}

#[test]
fn non_container_events_do_not_change_the_list() {
    assert!(!event(EventKind::Image, "delete").changes_container_list());
    assert!(!event(EventKind::Network, "create").changes_container_list());
}
