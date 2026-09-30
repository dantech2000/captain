//! The keys of the settings file as paths, such as `["kubernetes", "port"]`, and
//! which of them differ from the defaults. The file holds only those. See ADR 0013.

use serde_json::Value;

use super::Settings;

/// A key in the file: the top-level name, then the name inside a group such as
/// `kubernetes`.
pub(super) type KeyPath = Vec<String>;

/// The defaults as JSON.
pub(super) fn defaults() -> Value {
    to_value(&Settings::default())
}

pub(super) fn to_value(settings: &Settings) -> Value {
    serde_json::to_value(settings).unwrap_or_default()
}

/// Every key that holds one value. A group is an object with keys in the defaults,
/// such as `kubernetes`; its keys are listed instead. An object with no keys by
/// default, such as `engine_daemon.custom`, or `null`, such as `engine_resources`,
/// is one value. The file edit still changes such an object key by key, and the
/// check reads its fields against their own limits.
pub(super) fn leaves(defaults: &Value) -> Vec<KeyPath> {
    let mut paths = Vec::new();
    collect(defaults, &mut Vec::new(), &mut paths);
    paths
}

fn collect(value: &Value, prefix: &mut KeyPath, paths: &mut Vec<KeyPath>) {
    match value {
        Value::Object(map) if !map.is_empty() => {
            for (key, inner) in map {
                prefix.push(key.clone());
                collect(inner, prefix, paths);
                prefix.pop();
            }
        }
        _ => paths.push(prefix.clone()),
    }
}

/// The value at `path`, if `value` has it.
pub(super) fn get<'a>(value: &'a Value, path: &[String]) -> Option<&'a Value> {
    path.iter().try_fold(value, |value, key| value.get(key))
}

/// The keys whose values differ between `before` and `after`.
pub(super) fn changed(before: &Settings, after: &Settings) -> Vec<KeyPath> {
    let (before, after) = (to_value(before), to_value(after));
    leaves(&defaults())
        .into_iter()
        .filter(|path| get(&before, path) != get(&after, path))
        .collect()
}

/// `true` for the `version` key, which the file always holds.
pub(super) fn is_version(path: &[String]) -> bool {
    path.len() == 1 && path[0] == "version"
}
