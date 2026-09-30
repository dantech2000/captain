use bollard::models::{ContainerUpdateBody, RestartPolicy as EngineRestart};
use captain_core::model::ResourceUpdate;

use super::restart_name;

/// The body of `POST /containers/{id}/update`. Fields left `None` stay as they are.
pub fn update_body(update: ResourceUpdate) -> ContainerUpdateBody {
    let memory = update
        .memory
        .map(|bytes| i64::try_from(bytes).unwrap_or(i64::MAX));
    ContainerUpdateBody {
        memory,
        // As `docker run --memory` sets it, so a limit above the old swap is accepted.
        memory_swap: memory.map(|memory| memory.saturating_mul(2)),
        nano_cpus: update
            .nano_cpus
            .map(|nano| i64::try_from(nano).unwrap_or(i64::MAX)),
        restart_policy: update.restart_policy.map(|policy| EngineRestart {
            name: Some(restart_name(policy)),
            maximum_retry_count: None,
        }),
        ..ContainerUpdateBody::default()
    }
}

#[cfg(test)]
mod tests;
