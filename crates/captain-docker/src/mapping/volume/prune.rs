use std::collections::HashMap;

use bollard::models::VolumePruneResponse;
use captain_core::model::VolumePrune;

/// The `filters` for `POST /volumes/prune`. `all=true` makes API 1.42 and later
/// remove unused named volumes too, not only anonymous ones.
pub fn volume_prune_filters(all: bool, label: Option<&str>) -> HashMap<String, Vec<String>> {
    let mut filters = HashMap::new();
    if all {
        filters.insert("all".to_string(), vec!["true".to_string()]);
    }
    if let Some(label) = label {
        filters.insert("label".to_string(), vec![label.to_string()]);
    }
    filters
}

pub fn volume_prune(response: VolumePruneResponse) -> VolumePrune {
    VolumePrune {
        removed: response.volumes_deleted.unwrap_or_default(),
        reclaimed_bytes: response
            .space_reclaimed
            .and_then(|bytes| u64::try_from(bytes).ok())
            .unwrap_or_default(),
    }
}
