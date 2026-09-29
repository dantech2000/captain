use super::{changes_network_list, changes_volume_list};
use crate::model::{EngineEvent, EventKind};

fn event(kind: EventKind, action: &str) -> EngineEvent {
    EngineEvent {
        kind,
        action: action.into(),
        id: "x".into(),
    }
}

#[test]
fn volume_list_follows_volume_and_container_lifecycle_events() {
    assert!(changes_volume_list(&event(EventKind::Volume, "create")));
    assert!(changes_volume_list(&event(EventKind::Volume, "unmount")));
    assert!(changes_volume_list(&event(EventKind::Container, "destroy")));
    assert!(!changes_volume_list(&event(EventKind::Container, "start")));
    assert!(!changes_volume_list(&event(EventKind::Network, "create")));
}

#[test]
fn network_list_follows_network_events() {
    assert!(changes_network_list(&event(EventKind::Network, "connect")));
    assert!(changes_network_list(&event(EventKind::Network, "destroy")));
    assert!(!changes_network_list(&event(EventKind::Volume, "create")));
    assert!(!changes_network_list(&event(EventKind::Container, "start")));
}
