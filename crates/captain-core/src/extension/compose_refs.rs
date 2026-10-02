//! The files an extension's Compose file names by a relative path. When the Compose
//! file sits at the image root, Captain copies it and these files, not the whole
//! image. See <https://docs.docker.com/reference/compose-file/>.

use std::path::{Component, Path};

use serde_json::Value;

/// Compose reads this file next to the Compose file for its variables.
const DOT_ENV: &str = ".env";

/// The relative paths that `yaml` names: `.env`, `env_file`, bind mount sources,
/// `extends.file`, `include`, and the files of `configs` and `secrets`. Paths that
/// are absolute or leave the folder are left out. Sorted, without duplicates.
pub fn compose_references(yaml: &str) -> Vec<String> {
    let mut found = vec![DOT_ENV.to_string()];
    let Ok(file) = serde_saphyr::from_str::<Value>(yaml) else {
        return found;
    };
    found.extend(compose_files_in(&file));
    let mut add = |value: Option<&Value>| {
        found.extend(value.and_then(Value::as_str).and_then(relative));
    };
    for service in objects(file.get("services")) {
        for env in list(service.get("env_file")) {
            add(env.get("path").or(Some(env)));
        }
        for volume in list(service.get("volumes")) {
            match volume {
                Value::String(short) => {
                    let source = short.split(':').next().unwrap_or_default();
                    if source.starts_with('.') {
                        add(Some(&Value::String(source.to_string())));
                    }
                }
                long if long.get("type").and_then(Value::as_str) == Some("bind") => {
                    add(long.get("source"));
                }
                _ => {}
            }
        }
    }
    for section in ["configs", "secrets"] {
        for entry in objects(file.get(section)) {
            add(entry.get("file"));
        }
    }
    found.sort();
    found.dedup();
    found
}

/// The Compose files that `yaml` names by a relative path: `include` and
/// `extends.file`. Compose reads these too, so they can name more files.
pub(super) fn compose_files(yaml: &str) -> Vec<String> {
    serde_saphyr::from_str::<Value>(yaml)
        .map(|file| compose_files_in(&file))
        .unwrap_or_default()
}

fn compose_files_in(file: &Value) -> Vec<String> {
    let mut found = Vec::new();
    let mut add = |value: Option<&Value>| {
        found.extend(value.and_then(Value::as_str).and_then(relative));
    };
    for service in objects(file.get("services")) {
        add(service.get("extends").and_then(|e| e.get("file")));
    }
    for include in list(file.get("include")) {
        match include.get("path") {
            Some(paths) => list(Some(paths)).into_iter().for_each(|p| add(Some(p))),
            None => add(Some(include)),
        }
    }
    found
}

fn objects(value: Option<&Value>) -> Vec<&Value> {
    value
        .and_then(Value::as_object)
        .map(|map| map.values().collect())
        .unwrap_or_default()
}

/// A list, or one value as a list of one.
fn list(value: Option<&Value>) -> Vec<&Value> {
    match value {
        Some(Value::Array(items)) => items.iter().collect(),
        Some(Value::Null) | None => Vec::new(),
        Some(one) => vec![one],
    }
}

/// `path` without `./`, when it stays inside the Compose file's folder.
fn relative(path: &str) -> Option<String> {
    let mut parts = Vec::new();
    for component in Path::new(path).components() {
        match component {
            Component::Normal(part) => parts.push(part.to_str()?),
            Component::CurDir => {}
            _ => return None,
        }
    }
    (!parts.is_empty()).then(|| parts.join("/"))
}

#[cfg(test)]
mod tests;
