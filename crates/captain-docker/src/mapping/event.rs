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
    let actor = message.actor.unwrap_or_default();
    let mut attributes = actor.attributes.unwrap_or_default();
    EngineEvent {
        kind,
        action: message.action.unwrap_or_default(),
        id: actor.id.unwrap_or_default(),
        time: message.time,
        name: attributes.remove("name"),
        exit_code: attributes
            .get("exitCode")
            .and_then(|code| code.parse().ok()),
    }
}
