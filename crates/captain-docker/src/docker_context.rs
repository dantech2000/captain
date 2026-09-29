//! Reads the current context from the Docker CLI config in `~/.docker`.

use std::fs;
use std::path::Path;

use serde_json::Value;

/// Returns the engine host of the current Docker CLI context, if one is set.
///
/// `context_override` is the value of `DOCKER_CONTEXT`, which wins over the config file.
/// The `default` context has no metadata, so it returns `None`.
pub fn current_context_host(docker_dir: &Path, context_override: Option<&str>) -> Option<String> {
    let name = match context_override {
        Some(name) => name.to_string(),
        None => read_json(&docker_dir.join("config.json"))?
            .get("currentContext")?
            .as_str()?
            .to_string(),
    };
    if name.is_empty() || name == "default" {
        return None;
    }
    find_context_host(&docker_dir.join("contexts").join("meta"), &name)
}

/// The Docker CLI stores each context in `meta/<sha256 of name>/meta.json`.
/// Captain scans the directory and matches on `Name`, so it needs no hash library.
fn find_context_host(meta_dir: &Path, name: &str) -> Option<String> {
    fs::read_dir(meta_dir)
        .ok()?
        .flatten()
        .filter_map(|entry| read_json(&entry.path().join("meta.json")))
        .find(|meta| meta.get("Name").and_then(Value::as_str) == Some(name))?
        .pointer("/Endpoints/docker/Host")?
        .as_str()
        .map(ToString::to_string)
}

fn read_json(path: &Path) -> Option<Value> {
    serde_json::from_slice(&fs::read(path).ok()?).ok()
}

#[cfg(test)]
mod tests;
