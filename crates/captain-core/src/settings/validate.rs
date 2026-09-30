//! Reads the settings file leniently and names each value that this build cannot
//! use, with its line. See ADR 0013.

use std::collections::HashMap;

use serde::Deserialize;
use serde_json::Value;

use super::overrides::{defaults, get, leaves, to_value};
use super::{FileProblem, ReferenceEntry, Settings, jsonc, reference_entries};

/// The settings in `text`, and the values that fell back to their defaults. Bad
/// JSON, or a file that is not one object, is an error.
pub(super) fn read(text: &str) -> Result<(Settings, Vec<FileProblem>), FileProblem> {
    let file = jsonc::parse(text)?;
    if !file.is_object() {
        return Err(FileProblem {
            line: 1,
            key: None,
            message: "The file must hold one object, in { }".into(),
        });
    }
    let settings = Settings::from_value(file.clone());
    let problems = problems(text, &file, &settings);
    Ok((settings, problems))
}

fn problems(text: &str, file: &Value, settings: &Settings) -> Vec<FileProblem> {
    let read = to_value(settings);
    let entries: HashMap<String, ReferenceEntry> = reference_entries()
        .into_iter()
        .map(|entry| (entry.key.clone(), entry))
        .collect();
    let mut problems = Vec::new();
    let mut bad_groups: Vec<String> = Vec::new();
    for path in leaves(&defaults()) {
        let group = &path[..1];
        if path.len() > 1 && get(file, group).is_some_and(|value| !value.is_object()) {
            if !bad_groups.contains(&path[0]) {
                bad_groups.push(path[0].clone());
                problems.push(problem(text, group, "must be an object".into()));
            }
            continue;
        }
        let Some(value) = get(file, &path) else {
            continue;
        };
        let key = path.join(".");
        let entry = entries.get(&key);
        let kept = get(&read, &path).is_some_and(|read| same(read, value));
        if !kept || entry.is_some_and(|entry| out_of_range(entry, value)) {
            let expected = entry.map_or("a valid value", |entry| entry.type_label.as_str());
            problems.push(problem(text, &path, format!("must be {expected}")));
        }
    }
    problems
}

/// True if the file's `value` reads as `read`. Blank or padded text counts as
/// read, because Captain trims it.
fn same(read: &Value, value: &Value) -> bool {
    match (read, value.as_str().map(str::trim)) {
        _ if read == value => true,
        (Value::Null, Some("")) => true,
        (Value::String(read), Some(trimmed)) => read == trimmed,
        _ => false,
    }
}

fn out_of_range(entry: &ReferenceEntry, value: &Value) -> bool {
    let Some(number) = value.as_f64() else {
        return false;
    };
    entry.minimum.is_some_and(|min| number < min) || entry.maximum.is_some_and(|max| number > max)
}

fn problem(text: &str, path: &[String], message: String) -> FileProblem {
    FileProblem {
        line: jsonc::line_of(text, path).unwrap_or(1),
        key: Some(path.join(".")),
        message,
    }
}

impl Settings {
    /// Parses a settings file, with comments allowed. Only bad JSON is an error; a
    /// value this build cannot use gets its default.
    pub fn from_json(json: &str) -> Result<Self, FileProblem> {
        read(json).map(|(settings, _)| settings)
    }

    /// The settings in a parsed file, leniently.
    fn from_value(value: Value) -> Self {
        let mut settings = Settings::deserialize(value).unwrap_or_default();
        settings.engine_endpoint = settings
            .engine_endpoint
            .map(|host| host.trim().to_string())
            .filter(|host| !host.is_empty());
        settings
    }
}

#[cfg(test)]
mod tests;
