//! Which volumes and networks a prune removes. The engine decides for real; the fake
//! engine uses these rules, and they document what Captain expects.

use std::collections::BTreeMap;

use crate::model::{Network, Volume};

/// True if `labels` pass a Docker `label` filter: `key` needs the key, and
/// `key=value` needs the key with that value. No filter passes everything.
pub fn label_matches(labels: &BTreeMap<String, String>, filter: Option<&str>) -> bool {
    let Some(filter) = filter else {
        return true;
    };
    match filter.split_once('=') {
        Some((key, value)) => labels.get(key).is_some_and(|v| v == value),
        None => labels.contains_key(filter),
    }
}

/// Volumes that a prune removes: unused ones that pass `label`. Without `all`, only
/// anonymous ones, as on Docker API 1.42 and later. A volume with an unknown container
/// count is kept.
pub fn prunable_volumes<'a>(
    volumes: &'a [Volume],
    all: bool,
    label: Option<&str>,
) -> Vec<&'a Volume> {
    volumes
        .iter()
        .filter(|v| v.is_unused())
        .filter(|v| all || v.pruned_by_default())
        .filter(|v| label_matches(&v.labels, label))
        .collect()
}

/// Networks that a prune removes: ones that are not built-in, have no containers,
/// and pass `label`.
pub fn prunable_networks<'a>(networks: &'a [Network], label: Option<&str>) -> Vec<&'a Network> {
    networks
        .iter()
        .filter(|n| n.can_remove())
        .filter(|n| label_matches(&n.labels, label))
        .collect()
}

/// For example `Removed 3 networks`, or `No unused networks to remove`.
pub fn network_prune_summary(removed: &[String]) -> String {
    match removed.len() {
        0 => "No unused networks to remove".into(),
        1 => "Removed 1 network".into(),
        n => format!("Removed {n} networks"),
    }
}

#[cfg(test)]
mod tests;
