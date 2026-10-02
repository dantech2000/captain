//! Kubeconfig documents as JSON values: the `captain-desktop` entry made from k3s's
//! own file, the merge into the user's file, and the context list. Only entries named
//! `captain-desktop` ever change, and Captain's own entries from before the rename
//! (see `kubeconfig_legacy`); other clusters, users, and contexts stay as they are.
//! See ADR 0010.

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use serde_json::{Map, Value, json};

/// The name of Captain's cluster, user, and context, like `docker-desktop` and
/// `rancher-desktop`.
pub const CONTEXT: &str = "captain-desktop";

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

/// The `captain-desktop` config made from k3s's `k3s.yaml`: its first cluster and
/// user, renamed to `captain-desktop`, with the server at `https://127.0.0.1:<port>`.
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

/// The PEM certificate authority of `config`'s first cluster, to check that a
/// server on the host port is this cluster and not another program.
pub fn cluster_ca(config: &Value) -> Result<Vec<u8>, String> {
    let data = config["clusters"][0]["cluster"]["certificate-authority-data"]
        .as_str()
        .ok_or("k3s.yaml has no certificate authority.")?;
    STANDARD
        .decode(data)
        .map_err(|error| format!("k3s.yaml has a bad certificate authority: {error}"))
}

/// `existing` with its `captain-desktop` entries replaced by those in `captain`. The
/// current context becomes `captain-desktop` only when there is none.
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

/// `config` without the `captain-desktop` cluster, user, and context. A current
/// context of `captain-desktop` is cleared.
pub fn remove_captain(config: &Value) -> Value {
    let mut config = config.clone();
    if !config.is_object() {
        config = Value::Object(Map::new());
    }
    remove_named(&mut config, CONTEXT, "");
    config
}

/// Removes the cluster, user, and context `name` from `config`. A current context of
/// `name` becomes `current`.
pub(super) fn remove_named(config: &mut Value, name: &str, current: &str) {
    let map = object(config);
    for list in LISTS {
        if map.contains_key(list) {
            list_mut(map, list).retain(|entry| entry["name"] != name);
        }
    }
    if map.get("current-context").and_then(Value::as_str) == Some(name) {
        map.insert("current-context".into(), current.into());
    }
}

/// The base64 certificate authority of the `captain-desktop` cluster in `config`.
pub fn captain_ca_data(config: &Value) -> Option<String> {
    config["clusters"]
        .as_array()?
        .iter()
        .find(|entry| entry["name"] == CONTEXT)?["cluster"]["certificate-authority-data"]
        .as_str()
        .map(String::from)
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
