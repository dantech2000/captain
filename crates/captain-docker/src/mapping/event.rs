use bollard::models::{EventMessage, EventMessageTypeEnum};
use captain_core::model::{EngineEvent, EventKind};

pub fn event(message: EventMessage) -> EngineEvent {
    let kind = match message.typ {
        Some(EventMessageTypeEnum::CONTAINER) => EventKind::Container,
        Some(EventMessageTypeEnum::IMAGE) => EventKind::Image,
        Some(EventMessageTypeEnum::VOLUME) => EventKind::Volume,
        Some(EventMessageTypeEnum::NETWORK) => EventKind::Network,
        _ => EventKind::Other,
    };
    EngineEvent {
        kind,
        action: message.action.unwrap_or_default(),
        id: message.actor.and_then(|actor| actor.id).unwrap_or_default(),
    }
}
