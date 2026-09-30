//! Edits the text of `settings.json` in place, so its comments, key order, and
//! formatting stay. It uses the concrete syntax tree of `jsonc-parser`
//! (https://docs.rs/jsonc-parser/0.34.0/jsonc_parser/cst/index.html). See ADR 0013.

use jsonc_parser::cst::{CstInputValue, CstObject, CstRootNode};
use serde_json::Value;

use super::overrides::{self, KeyPath, get, is_version, leaves};
use super::{FileProblem, SETTINGS_VERSION, Settings, jsonc};

/// A new file: a note, the schema for editors, and the format version.
pub(super) fn new_file() -> String {
    format!(
        "{{\n  // Only the settings you change. Every option and its default: Settings > All options,\n  \
         // or {url}\n  \"$schema\": \"./settings.schema.json\",\n  \"version\": {SETTINGS_VERSION}\n}}\n",
        url = super::REFERENCE_URL
    )
}

/// A file open for edits.
pub(super) struct FileEdit {
    root: CstRootNode,
    defaults: Value,
}

impl FileEdit {
    /// Opens `text`, or a new file when `text` is blank. Bad JSON is an error, so a
    /// file that the user can still fix is never replaced.
    pub(super) fn open(text: &str) -> Result<Self, FileProblem> {
        let text = if text.trim().is_empty() {
            new_file()
        } else {
            text.to_string()
        };
        jsonc::parse(&text)?;
        let root = CstRootNode::parse(&text, &jsonc::options()).map_err(|error| FileProblem {
            line: error.line_display(),
            key: None,
            message: error.kind().to_string(),
        })?;
        Ok(Self {
            root,
            defaults: overrides::defaults(),
        })
    }

    /// The file's current value.
    fn value(&self) -> Value {
        jsonc::parse(&self.root.to_string()).unwrap_or_default()
    }

    /// True when the file comes from a version before [`SETTINGS_VERSION`] 2, which
    /// wrote every key.
    pub(super) fn needs_migration(&self) -> bool {
        self.value()
            .get("version")
            .and_then(Value::as_u64)
            .is_some_and(|version| version < 2)
    }

    /// Drops every key whose value is the default and adds the `$schema` line.
    /// Other keys, comments included, stay.
    pub(super) fn migrate(&mut self) {
        let file = self.value();
        let root = self.root.object_value_or_set();
        if root.get("$schema").is_none() {
            root.insert(0, "$schema", "./settings.schema.json".into());
        }
        for path in leaves(&self.defaults) {
            let value = get(&file, &path);
            if !is_version(&path) && value.is_some() && value == get(&self.defaults, &path) {
                self.remove(&path);
            }
        }
        self.stamp_version();
    }

    /// Writes the keys in `paths` from `after`: a default removes the key, and any
    /// other value replaces it in place or adds it at the end of its object.
    pub(super) fn apply(&mut self, after: &Settings, paths: &[KeyPath]) {
        let after = overrides::to_value(after);
        for path in paths.iter().filter(|path| !is_version(path)) {
            match get(&after, path) {
                Some(value) if get(&self.defaults, path) != Some(value) => self.set(path, value),
                _ => self.remove(path),
            }
        }
        self.stamp_version();
    }

    fn stamp_version(&self) {
        self.set(&["version".to_string()], &Value::from(SETTINGS_VERSION));
    }

    pub(super) fn text(&self) -> String {
        self.root.to_string()
    }

    fn set(&self, path: &[String], value: &Value) {
        let Some((last, parents)) = path.split_last() else {
            return;
        };
        let object = parents
            .iter()
            .fold(self.root.object_value_or_set(), |object, key| {
                object.object_value_or_set(key)
            });
        match object.get(last) {
            Some(prop) => prop.set_value(input(value)),
            None => {
                object.append(last, input(value));
            }
        }
    }

    /// Removes the key at `path`, and a group that it leaves empty.
    fn remove(&self, path: &[String]) {
        let Some(root) = self.root.object_value() else {
            return;
        };
        let mut chain: Vec<CstObject> = vec![root];
        for key in &path[..path.len().saturating_sub(1)] {
            match chain.last().and_then(|object| object.object_value(key)) {
                Some(object) => chain.push(object),
                None => return,
            }
        }
        let Some(last) = path.last() else {
            return;
        };
        if let Some(prop) = chain.last().and_then(|object| object.get(last)) {
            prop.remove();
        }
        // Drop the groups the removal emptied, from the innermost out.
        for depth in (1..chain.len()).rev() {
            if !chain[depth].properties().is_empty() {
                break;
            }
            if let Some(prop) = chain[depth - 1].get(&path[depth - 1]) {
                prop.remove();
            }
        }
    }
}

/// `value` for the syntax tree.
fn input(value: &Value) -> CstInputValue {
    match value {
        Value::Null => CstInputValue::Null,
        Value::Bool(b) => CstInputValue::Bool(*b),
        Value::Number(n) => CstInputValue::Number(n.to_string()),
        Value::String(s) => CstInputValue::String(s.clone()),
        Value::Array(items) => CstInputValue::Array(items.iter().map(input).collect()),
        Value::Object(map) => {
            CstInputValue::Object(map.iter().map(|(k, v)| (k.clone(), input(v))).collect())
        }
    }
}

#[cfg(test)]
mod tests;
