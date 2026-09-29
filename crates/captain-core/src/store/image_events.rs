use crate::model::{EngineEvent, EventKind};

/// Image actions that add, remove, or rename images.
const IMAGE_ACTIONS: &[&str] = &["pull", "tag", "untag", "delete", "import", "load", "prune"];

/// Container actions that change how many containers use an image.
const CONTAINER_ACTIONS: &[&str] = &["create", "destroy"];

/// True if the image list must reload after `event`.
pub fn changes_image_list(event: &EngineEvent) -> bool {
    let action = event.action.as_str();
    match event.kind {
        EventKind::Image => IMAGE_ACTIONS.contains(&action),
        EventKind::Container => CONTAINER_ACTIONS.contains(&action),
        _ => false,
    }
}

#[cfg(test)]
mod tests;
