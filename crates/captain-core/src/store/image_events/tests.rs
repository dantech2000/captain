use super::changes_image_list;
use crate::model::{EngineEvent, EventKind};

fn event(kind: EventKind, action: &str) -> EngineEvent {
    EngineEvent {
        kind,
        action: action.into(),
        id: "x".into(),
        ..EngineEvent::default()
    }
}

#[test]
fn image_changes_reload_the_list() {
    for action in ["pull", "tag", "untag", "delete", "load"] {
        assert!(
            changes_image_list(&event(EventKind::Image, action)),
            "{action}"
        );
    }
}

#[test]
fn container_create_and_destroy_change_usage() {
    assert!(changes_image_list(&event(EventKind::Container, "create")));
    assert!(changes_image_list(&event(EventKind::Container, "destroy")));
    assert!(!changes_image_list(&event(EventKind::Container, "start")));
}

#[test]
fn other_events_do_not_reload() {
    assert!(!changes_image_list(&event(EventKind::Image, "push")));
    assert!(!changes_image_list(&event(EventKind::Volume, "create")));
    assert!(!changes_image_list(&event(EventKind::Other, "delete")));
}
