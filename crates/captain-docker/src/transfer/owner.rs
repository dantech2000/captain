//! Marks the volumes and containers that Captain creates in the target with the
//! engine they came from. A switch-over replaces only what carries this mark, so it
//! never empties or starts something that was already in the target.

use std::collections::HashMap;

use captain_core::EngineError;

use super::source::SourceEngine;

/// The label on each volume and container that Captain copied into the target. Its
/// value is the source engine's ID.
pub const MIGRATED_FROM_LABEL: &str = "dev.captain.migrated-from";

/// The value of [`MIGRATED_FROM_LABEL`] for copies from `source`.
pub async fn origin(source: &SourceEngine) -> Result<String, EngineError> {
    Ok(source.engine_id().await?.unwrap_or_default())
}

/// `labels` with the mark for `origin` added.
pub fn mark(labels: Option<HashMap<String, String>>, origin: &str) -> HashMap<String, String> {
    let mut labels = labels.unwrap_or_default();
    labels.insert(MIGRATED_FROM_LABEL.into(), origin.into());
    labels
}

/// True if `labels` mark a copy that Captain made from `origin`.
pub fn is_copy_from(labels: Option<&HashMap<String, String>>, origin: &str) -> bool {
    labels
        .and_then(|labels| labels.get(MIGRATED_FROM_LABEL))
        .is_some_and(|value| value == origin)
}
