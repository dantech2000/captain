//! Options of `docker.listContainers`, `docker.listImages`, and `showOpenDialog`.

use std::collections::BTreeMap;

use serde_json::Value;

/// The options of the list calls, as the Engine API takes them.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ListOptions {
    pub all: bool,
    pub limit: Option<i32>,
    pub size: bool,
    pub digests: bool,
    pub filters: BTreeMap<String, Vec<String>>,
}

impl ListOptions {
    /// Reads the options object. `filters` is a JSON string, as the SDK documents,
    /// or an object. A filter value is a list, or a map from value to `true`, as the
    /// Engine API accepts both.
    pub fn parse(params: &Value) -> Result<Self, String> {
        let flag = |key: &str| params.get(key).and_then(Value::as_bool) == Some(true);
        let filters = match params.get("filters") {
            None | Some(Value::Null) => Value::Null,
            Some(Value::String(json)) => serde_json::from_str(json)
                .map_err(|error| format!("\"filters\" is not JSON: {error}"))?,
            Some(other) => other.clone(),
        };
        Ok(Self {
            all: flag("all"),
            limit: params
                .get("limit")
                .and_then(Value::as_i64)
                .and_then(|limit| i32::try_from(limit).ok()),
            size: flag("size"),
            digests: flag("digests"),
            filters: parse_filters(&filters)?,
        })
    }
}

fn parse_filters(filters: &Value) -> Result<BTreeMap<String, Vec<String>>, String> {
    let Some(filters) = filters.as_object() else {
        return match filters {
            Value::Null => Ok(BTreeMap::new()),
            _ => Err("\"filters\" must be an object".into()),
        };
    };
    filters
        .iter()
        .map(|(key, values)| {
            let values = match values {
                Value::Array(list) => list.iter().filter_map(text).collect(),
                Value::Object(map) => map
                    .iter()
                    .filter(|(_, on)| on.as_bool() == Some(true))
                    .map(|(value, _)| value.clone())
                    .collect(),
                other => text(other).into_iter().collect(),
            };
            Ok((key.clone(), values))
        })
        .collect()
}

/// A string, or a number or boolean written out, for filters like `dangling: [true]`.
fn text(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => Some(text.clone()),
        Value::Bool(_) | Value::Number(_) => Some(value.to_string()),
        _ => None,
    }
}

/// What the native open panel may pick, from Electron-style `properties`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpenDialogOptions {
    pub files: bool,
    pub directories: bool,
    pub multiple: bool,
}

impl OpenDialogOptions {
    /// `openFile`, `openDirectory`, and `multiSelections`. With neither `openFile`
    /// nor `openDirectory`, the panel picks files, as in Electron.
    pub fn parse(params: &Value) -> Self {
        let properties: Vec<&str> = params
            .get("properties")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .collect();
        let has = |name| properties.contains(&name);
        let directories = has("openDirectory");
        Self {
            files: has("openFile") || !directories,
            directories,
            multiple: has("multiSelections"),
        }
    }
}

#[cfg(test)]
mod tests;
