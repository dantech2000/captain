//! Every key of `settings.json` with its type, default, description, and example,
//! read from the schema. The in-app "All options" list and
//! docs/reference/settings.md both use it. See ADR 0013.

use serde_json::Value;

use super::overrides::defaults;
use super::settings_schema;

/// The headings of the reference, in order.
pub const GROUPS: [&str; 9] = [
    "Appearance",
    "Engine",
    "Docker daemon",
    "Kubernetes",
    "Startup",
    "Terminal",
    "AI agents",
    "Storage",
    "Diagnostics",
];

/// One key of the settings file.
#[derive(Debug, Clone, PartialEq)]
pub struct ReferenceEntry {
    /// The key, with a dot inside a group: `theme`, `kubernetes.port`.
    pub key: String,
    /// One of [`GROUPS`].
    pub group: &'static str,
    /// The values it takes, such as "one of system, light, dark" or "true or false".
    pub type_label: String,
    /// The default as JSON, such as `"system"` or `6443`.
    pub default: String,
    pub description: String,
    /// A value as JSON, such as `"dark"`.
    pub example: Option<String>,
    pub minimum: Option<f64>,
    pub maximum: Option<f64>,
}

/// The keys in [`GROUPS`] order, and in file order within a group. `version` and
/// `$schema` are not listed; Captain writes them.
pub fn reference_entries() -> Vec<ReferenceEntry> {
    let schema = settings_schema();
    let defaults = defaults();
    let mut entries = Vec::new();
    let Some(properties) = schema.get("properties").and_then(Value::as_object) else {
        return entries;
    };
    for (name, property) in properties {
        let Some(group) = property
            .get("x-captain-group")
            .and_then(Value::as_str)
            .and_then(|group| GROUPS.into_iter().find(|known| *known == group))
        else {
            continue;
        };
        let is_group = defaults
            .get(name)
            .and_then(Value::as_object)
            .is_some_and(|map| !map.is_empty());
        let inner = resolve(&schema, property);
        match inner.get("properties").and_then(Value::as_object) {
            Some(children) if is_group => {
                for (child, schema_node) in children {
                    let key = format!("{name}.{child}");
                    entries.push(entry(&schema, key, group, schema_node));
                }
            }
            _ => entries.push(entry(&schema, name.clone(), group, property)),
        }
    }
    entries.sort_by_key(|entry| GROUPS.iter().position(|group| *group == entry.group));
    entries
}

/// The fields of each setting that holds a structured value but is not a group,
/// such as `engine_resources.cpus`, with their limits. They are not in the
/// reference, which describes such a setting as a whole; the file check reads
/// each field against its own limits.
pub(super) fn field_entries() -> Vec<ReferenceEntry> {
    let schema = settings_schema();
    let defaults = defaults();
    let mut entries = Vec::new();
    let Some(properties) = schema.get("properties").and_then(Value::as_object) else {
        return entries;
    };
    for (name, property) in properties {
        let is_group = defaults
            .get(name)
            .and_then(Value::as_object)
            .is_some_and(|map| !map.is_empty());
        let group = GROUPS
            .into_iter()
            .find(|group| property.get("x-captain-group").and_then(Value::as_str) == Some(group));
        let (Some(group), false) = (group, is_group) else {
            continue;
        };
        let Some(fields) = object_schema(&schema, property)
            .get("properties")
            .and_then(Value::as_object)
        else {
            continue;
        };
        for (field, node) in fields {
            entries.push(entry(&schema, format!("{name}.{field}"), group, node));
        }
    }
    entries
}

/// The object schema of `node`, also inside an `anyOf` with `null`, as an
/// `Option` of a struct has.
fn object_schema<'a>(schema: &'a Value, node: &'a Value) -> &'a Value {
    let node = resolve(schema, node);
    let any = node.get("anyOf").and_then(Value::as_array);
    any.and_then(|any| {
        any.iter()
            .map(|item| resolve(schema, item))
            .find(|item| item.get("properties").is_some())
    })
    .unwrap_or(node)
}

fn entry(schema: &Value, key: String, group: &'static str, node: &Value) -> ReferenceEntry {
    let resolved = resolve(schema, node);
    let text = |value: &Value| serde_json::to_string(value).unwrap_or_default();
    let number = |name: &str| resolved.get(name).and_then(Value::as_f64);
    ReferenceEntry {
        key,
        group,
        type_label: type_label(schema, node),
        default: node
            .get("default")
            .map(text)
            .unwrap_or_else(|| "null".into()),
        description: node
            .get("description")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        example: node
            .get("examples")
            .and_then(|examples| examples.get(0))
            .map(text),
        minimum: number("minimum"),
        maximum: number("maximum"),
    }
}

/// `node`, or what its `$ref` (alone or as the one item of `allOf`) points to.
fn resolve<'a>(schema: &'a Value, node: &'a Value) -> &'a Value {
    let reference = node.get("$ref").or_else(|| {
        let all = node.get("allOf")?.as_array()?;
        (all.len() == 1).then(|| all[0].get("$ref")).flatten()
    });
    reference
        .and_then(Value::as_str)
        .and_then(|reference| reference.strip_prefix("#"))
        .and_then(|pointer| schema.pointer(pointer))
        .unwrap_or(node)
}

/// The values a schema node takes, in words.
fn type_label(schema: &Value, node: &Value) -> String {
    let node = resolve(schema, node);
    if let Some(values) = constants(node) {
        return format!("one of {}", values.join(", "));
    }
    if let Some(any) = node.get("anyOf").and_then(Value::as_array) {
        let labels: Vec<String> = any.iter().map(|item| type_label(schema, item)).collect();
        return labels.join(", or ");
    }
    let types: Vec<&str> = match node.get("type") {
        Some(Value::String(one)) => vec![one],
        Some(Value::Array(many)) => many.iter().filter_map(Value::as_str).collect(),
        _ => return "any JSON value".into(),
    };
    let labels: Vec<String> = types
        .iter()
        .map(|kind| one_type(schema, node, kind))
        .collect();
    labels.join(", or ")
}

fn one_type(schema: &Value, node: &Value, kind: &str) -> String {
    let number = |name: &str| node.get(name).and_then(Value::as_u64);
    match kind {
        "boolean" => "true or false".into(),
        "string" => "a string".into(),
        "null" => "null".into(),
        "integer" => match (number("minimum"), number("maximum")) {
            (Some(min), Some(max)) => format!("a whole number from {min} to {max}"),
            (Some(min), None) if min > 0 => format!("a whole number, {min} or more"),
            _ => "a whole number".into(),
        },
        "array" => match node.get("items").map(|items| type_label(schema, items)) {
            Some(item) if item == "a string" => "a list of strings".into(),
            Some(item) if item.starts_with("one of ") => {
                format!("a list of any of {}", &item["one of ".len()..])
            }
            _ => "a list".into(),
        },
        "object" => match node.get("required").and_then(Value::as_array) {
            Some(keys) if !keys.is_empty() => {
                let keys: Vec<&str> = keys.iter().filter_map(Value::as_str).collect();
                match keys.split_last() {
                    Some((last, [])) => format!("an object with {last}"),
                    Some((last, rest)) => format!("an object with {}, and {last}", rest.join(", ")),
                    None => "an object".into(),
                }
            }
            _ => "an object".into(),
        },
        other => other.into(),
    }
}

/// The string values of an `enum`, or of a `oneOf` of `const`s.
fn constants(node: &Value) -> Option<Vec<String>> {
    if let Some(values) = node.get("enum").and_then(Value::as_array) {
        return values
            .iter()
            .map(|v| v.as_str().map(String::from))
            .collect();
    }
    let one_of = node.get("oneOf")?.as_array()?;
    one_of
        .iter()
        .map(|item| {
            let value = item.get("const").or_else(|| item.get("enum")?.get(0))?;
            value.as_str().map(String::from)
        })
        .collect()
}
