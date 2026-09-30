//! The entries the All options sheet lists: every setting from the generated
//! reference (the same text as docs/reference/settings.md), with its value now.

use std::collections::HashMap;

use captain_core::settings::{Settings, reference_entries};
use serde_json::Value;

/// One setting: its dotted key, such as `kubernetes.port`, its value now, its
/// default, and what it does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptionEntry {
    pub key: String,
    pub value: String,
    pub default: String,
    pub description: String,
}

impl OptionEntry {
    /// True if the key or the value has `query`, ignoring case.
    pub fn matches(&self, query: &str) -> bool {
        let query = query.trim().to_lowercase();
        query.is_empty()
            || self.key.to_lowercase().contains(&query)
            || self.value.to_lowercase().contains(&query)
            || self.description.to_lowercase().contains(&query)
    }
}

/// Every setting in the reference, with its value in `settings`.
pub fn option_entries(settings: &Settings) -> Vec<OptionEntry> {
    let mut current = HashMap::new();
    if let Ok(value) = serde_json::to_value(settings) {
        flatten("", &value, &mut current);
    }
    reference_entries()
        .into_iter()
        .map(|entry| OptionEntry {
            value: current
                .remove(&entry.key)
                .unwrap_or_else(|| entry.default.clone()),
            key: entry.key,
            default: entry.default,
            description: entry.description,
        })
        .collect()
}

/// The values of `value` by dotted key, such as `kubernetes.port`.
fn flatten(prefix: &str, value: &Value, entries: &mut HashMap<String, String>) {
    match value {
        Value::Object(map) if !map.is_empty() => {
            for (key, value) in map {
                let key = match prefix {
                    "" => key.clone(),
                    _ => format!("{prefix}.{key}"),
                };
                flatten(&key, value, entries);
            }
        }
        _ if prefix == "version" => {}
        other => {
            entries.insert(prefix.to_string(), other.to_string());
        }
    }
}

#[cfg(test)]
mod tests;
