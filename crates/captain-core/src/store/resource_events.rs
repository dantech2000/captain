use crate::model::{EngineEvent, EventKind};

/// Container actions that change which containers use a volume.
const VOLUME_CONTAINER_ACTIONS: &[&str] = &["create", "destroy"];

/// True if the volume list must reload after this event. Volume events cover create,
/// destroy, mount, and unmount. A new or removed container changes the use counts.
pub fn changes_volume_list(event: &EngineEvent) -> bool {
    match event.kind {
        EventKind::Volume => true,
        EventKind::Container => VOLUME_CONTAINER_ACTIONS.contains(&event.action.as_str()),
        _ => false,
    }
}

/// True if the network list must reload after this event. Network events cover create,
/// destroy, connect, and disconnect, so they also track attached containers.
pub fn changes_network_list(event: &EngineEvent) -> bool {
    event.kind == EventKind::Network
}

#[cfg(test)]
mod tests;
