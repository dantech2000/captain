//! Kubeconfig documents as JSON values: the `captain` entry made from k3s's own
//! file, the merge into the user's file, and the context list. Only entries named
//! `captain` ever change; other clusters, users, and contexts stay as they are.
//! See ADR 0010.

use serde_json::{Map, Value, json};

/// The name of Captain's cluster, user, and context.
pub const CONTEXT: &str = "captain";

/// The lists in a kubeconfig whose entries have a `name`.
const LISTS: [&str; 3] = ["clusters", "users", "contexts"];

/// Parses a kubeconfig. Empty text is an empty config.
pub fn parse(yaml: &str) -> Result<Value, String> {
    if yaml.trim().is_empty() {
        return Ok(Value::Object(Map::new()));
    }
    let value: Value = serde_saphyr::from_str(yaml).map_err(|error| error.to_string())?;
    match value {
        Value::Object(_) => Ok(value),
        Value::Null => Ok(Value::Object(Map::new())),
        _ => Err("The kubeconfig is not a YAML mapping.".into()),
    }
}

pub fn to_yaml(config: &Value) -> Result<String, String> {
    serde_saphyr::to_string(config).map_err(|error| error.to_string())
}

/// The `captain` config made from k3s's `k3s.yaml`: its first cluster and user,
/// renamed to `captain`, with the server at `https://127.0.0.1:<port>`.
pub fn captain_config(k3s_yaml: &str, port: u16) -> Result<Value, String> {
    let k3s = parse(k3s_yaml)?;
    let first = |list: &str, key: &str| {
        k3s[list][0][key]
            .as_object()
            .cloned()
            .ok_or_else(|| format!("k3s.yaml has no {list}."))
    };
    let mut cluster = first("clusters", "cluster")?;
    cluster.insert("server".into(), format!("https://127.0.0.1:{port}").into());
    let user = first("users", "user")?;
    Ok(json!({
        "apiVersion": "v1",
        "kind": "Config",
        "clusters": [{"name": CONTEXT, "cluster": cluster}],
        "users": [{"name": CONTEXT, "user": user}],
        "contexts": [{"name": CONTEXT, "context": {"cluster": CONTEXT, "user": CONTEXT}}],
        "current-context": CONTEXT,
    }))
}

/// `existing` with its `captain` entries replaced by those in `captain`. The current
/// context becomes `captain` only when there is none.
pub fn merge(existing: &Value, captain: &Value) -> Value {
    let mut config = remove_captain(existing);
    let map = object(&mut config);
    map.entry("apiVersion").or_insert_with(|| "v1".into());
    map.entry("kind").or_insert_with(|| "Config".into());
    for list in LISTS {
        let entries = captain[list].as_array().cloned().unwrap_or_default();
        list_mut(map, list).extend(entries);
    }
    if current_context(&config).is_none() {
        object(&mut config).insert("current-context".into(), CONTEXT.into());
    }
    config
}

/// `config` without the `captain` cluster, user, and context. A current context of
/// `captain` is cleared.
pub fn remove_captain(config: &Value) -> Value {
    let mut config = config.clone();
    if !config.is_object() {
        config = Value::Object(Map::new());
    }
    let map = object(&mut config);
    for list in LISTS {
        if map.contains_key(list) {
            list_mut(map, list).retain(|entry| entry["name"] != CONTEXT);
        }
    }
    if map.get("current-context").and_then(Value::as_str) == Some(CONTEXT) {
        map.insert("current-context".into(), "".into());
    }
    config
}

/// The context names, in file order.
pub fn contexts(config: &Value) -> Vec<String> {
    config["contexts"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|entry| entry["name"].as_str().map(String::from))
        .collect()
}

/// The current context, or `None` when it is missing or empty.
pub fn current_context(config: &Value) -> Option<String> {
    config["current-context"]
        .as_str()
        .filter(|name| !name.is_empty())
        .map(String::from)
}

pub fn set_current(config: &Value, name: &str) -> Value {
    let mut config = config.clone();
    object(&mut config).insert("current-context".into(), name.into());
    config
}

fn object(value: &mut Value) -> &mut Map<String, Value> {
    if !value.is_object() {
        *value = Value::Object(Map::new());
    }
    value.as_object_mut().expect("the value is an object")
}

/// The list `key` of `map`; a missing or `null` list becomes empty.
fn list_mut<'a>(map: &'a mut Map<String, Value>, key: &str) -> &'a mut Vec<Value> {
    let entry = map.entry(key).or_insert_with(|| Value::Array(Vec::new()));
    if !entry.is_array() {
        *entry = Value::Array(Vec::new());
    }
    entry.as_array_mut().expect("the value is an array")
}

#[cfg(test)]
mod tests;
