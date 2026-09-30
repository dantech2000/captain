use std::sync::OnceLock;

use serde_json::Value;

/// The Compose schema that Compose v5.5.1 validates against: compose-go v2.15.0's
/// `schema/compose-spec.json`, Apache-2.0, vendored with its LICENSE and NOTICE.
/// Update it when `COMPOSE_VERSION` in scripts/tool-versions.env changes.
const COMPOSE_SPEC: &str = include_str!("../../vendor/compose-go/compose-spec.json");

/// Captain's own top-level key, `x-captain`, with its tasks.
const X_CAPTAIN: &str = include_str!("x_captain.json");

/// How many `$ref`s in a row Captain follows before it gives up.
const MAX_REFS: usize = 8;

/// A key the schema allows, with its description for completion and hover.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaKey {
    pub name: String,
    pub doc: Option<String>,
}

/// Keys and descriptions from the Compose schema, plus `x-captain`.
pub struct ComposeSchema {
    root: Value,
}

impl ComposeSchema {
    /// The bundled schema, read once.
    pub fn bundled() -> &'static Self {
        static SCHEMA: OnceLock<ComposeSchema> = OnceLock::new();
        SCHEMA.get_or_init(|| Self::parse(COMPOSE_SPEC, X_CAPTAIN))
    }

    fn parse(spec: &str, x_captain: &str) -> Self {
        let mut root: Value = serde_json::from_str(spec).unwrap_or_default();
        if let (Some(properties), Ok(extra)) = (
            root.get_mut("properties").and_then(Value::as_object_mut),
            serde_json::from_str::<Value>(x_captain),
        ) {
            properties.insert("x-captain".into(), extra);
        }
        Self { root }
    }

    /// The keys allowed in the mapping at `parents`, sorted by name. A list item's
    /// index, such as `0`, stands for any item.
    pub fn keys(&self, parents: &[String]) -> Vec<SchemaKey> {
        let mut keys: Vec<SchemaKey> = Vec::new();
        for node in self.nodes_at(parents) {
            let properties = node.get("properties").and_then(Value::as_object);
            for (name, property) in properties.into_iter().flatten() {
                if keys.iter().all(|key| &key.name != name) {
                    keys.push(SchemaKey {
                        name: name.clone(),
                        doc: self.description(property),
                    });
                }
            }
        }
        keys.sort_by(|a, b| a.name.cmp(&b.name));
        keys
    }

    /// The description of the key at `path`, for example `services.web.image`.
    pub fn doc(&self, path: &[String]) -> Option<String> {
        let (last, parents) = path.split_last()?;
        self.nodes_at(parents)
            .into_iter()
            .filter_map(|node| child(node, last))
            .find_map(|node| self.description(node))
    }

    /// The schema nodes that may describe the value at `path`, with `$ref`s
    /// followed and `oneOf`, `anyOf`, and `allOf` spread out.
    fn nodes_at(&self, path: &[String]) -> Vec<&Value> {
        let mut nodes = self.expand(&self.root);
        for segment in path {
            nodes = nodes
                .into_iter()
                .filter_map(|node| child(node, segment))
                .flat_map(|node| self.expand(node))
                .collect();
        }
        nodes
    }

    fn expand<'a>(&'a self, node: &'a Value) -> Vec<&'a Value> {
        let mut out = Vec::new();
        self.expand_into(node, 0, &mut out);
        out
    }

    fn expand_into<'a>(&'a self, node: &'a Value, depth: usize, out: &mut Vec<&'a Value>) {
        if depth > MAX_REFS {
            return;
        }
        if let Some(target) = self.resolve(node) {
            self.expand_into(target, depth + 1, out);
        }
        out.push(node);
        for key in ["oneOf", "anyOf", "allOf"] {
            for alternative in node
                .get(key)
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                self.expand_into(alternative, depth + 1, out);
            }
        }
    }

    /// The node a `$ref` such as `#/$defs/service` points at.
    fn resolve(&self, node: &Value) -> Option<&Value> {
        let reference = node.get("$ref")?.as_str()?;
        self.root.pointer(reference.strip_prefix('#')?)
    }

    /// A node's own description, or else the one of what it refers to.
    fn description(&self, node: &Value) -> Option<String> {
        let text = |node: &Value| {
            node.get("description")
                .and_then(Value::as_str)
                .map(String::from)
        };
        text(node).or_else(|| self.expand(node).into_iter().find_map(text))
    }
}

/// The node for `segment` under `node`: a named property, a list's items for an
/// index, a pattern property (`^x-` for extension keys), or additional properties.
fn child<'a>(node: &'a Value, segment: &str) -> Option<&'a Value> {
    if let Some(property) = node.pointer(&format!("/properties/{}", escape(segment))) {
        return Some(property);
    }
    if segment.bytes().all(|b| b.is_ascii_digit())
        && let Some(items) = node.get("items")
    {
        return Some(items);
    }
    if let Some(patterns) = node.get("patternProperties").and_then(Value::as_object) {
        let extension = segment.starts_with("x-");
        if let Some((_, schema)) = patterns
            .iter()
            .find(|(pattern, _)| (pattern.as_str() == "^x-") == extension)
        {
            return Some(schema);
        }
    }
    node.get("additionalProperties")
        .filter(|value| value.is_object())
}

/// A JSON Pointer segment: `~` and `/` escaped.
fn escape(segment: &str) -> String {
    segment.replace('~', "~0").replace('/', "~1")
}

#[cfg(test)]
mod tests;
