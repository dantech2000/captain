//! Builds `daemon.json` from [`DaemonSettings`]. The rules, in order:
//!
//! 1. Start with the custom keys, without the keys Captain manages.
//! 2. Set `registry-mirrors` and `insecure-registries` from their fields, when not empty.
//! 3. Merge `features` key by key; `cdi` and `containerd-snapshotter` stay `true`,
//!    because the Lima template sets them on every boot.

use serde_json::{Map, Value};

use super::DaemonSettings;

/// Keys the custom JSON cannot set, and why. `features.<name>` means a key inside
/// the `features` object.
pub const MANAGED_KEYS: &[(&str, &str)] = &[
    (
        "hosts",
        "Captain sets the listeners. Use the TCP switch instead.",
    ),
    (
        "containerd",
        "Captain Engine's Docker service sets it already.",
    ),
    ("registry-mirrors", "Use the Registry mirrors field."),
    ("insecure-registries", "Use the Insecure registries field."),
    ("features.cdi", "Captain Engine keeps it on."),
    (
        "features.containerd-snapshotter",
        "Captain Engine keeps it on; the images are in the containerd store.",
    ),
];

/// The features Captain Engine always turns on.
const FEATURES: &[&str] = &["cdi", "containerd-snapshotter"];

/// Fails with a message if `custom` sets a key that Captain manages.
pub fn check_custom(custom: &Map<String, Value>) -> Result<(), String> {
    let features = custom.get("features").and_then(Value::as_object);
    for (key, why) in MANAGED_KEYS {
        let set = match key.strip_prefix("features.") {
            Some(feature) => features.is_some_and(|f| f.contains_key(feature)),
            None => custom.contains_key(*key),
        };
        if set {
            return Err(format!("Captain manages \"{key}\". {why}"));
        }
    }
    Ok(())
}

/// The `daemon.json` object for `settings`.
pub fn daemon_json(settings: &DaemonSettings) -> Value {
    let mut json = settings.custom.clone();
    for (key, _) in MANAGED_KEYS {
        if !key.contains('.') {
            json.remove(*key);
        }
    }
    let list = |items: &[String]| Value::from(items.to_vec());
    if !settings.registry_mirrors.is_empty() {
        json.insert("registry-mirrors".into(), list(&settings.registry_mirrors));
    }
    if !settings.insecure_registries.is_empty() {
        json.insert(
            "insecure-registries".into(),
            list(&settings.insecure_registries),
        );
    }
    let mut features = match json.remove("features") {
        Some(Value::Object(features)) => features,
        _ => Map::new(),
    };
    for feature in FEATURES {
        features.insert((*feature).into(), Value::Bool(true));
    }
    json.insert("features".into(), Value::Object(features));
    Value::Object(json)
}

#[cfg(test)]
mod tests;
