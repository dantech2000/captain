use std::path::{Path, PathBuf};

/// Compose's file names, in the order it tries them (compose-go's
/// `DefaultFileNames`).
const COMPOSE_NAMES: [&str; 4] = [
    "compose.yaml",
    "compose.yml",
    "docker-compose.yml",
    "docker-compose.yaml",
];

/// Compose's override file names, in its order (`DefaultOverrideFileNames`).
const OVERRIDE_NAMES: [&str; 4] = [
    "compose.override.yml",
    "compose.override.yaml",
    "docker-compose.override.yml",
    "docker-compose.override.yaml",
];

/// The Compose files `docker compose` would use in `dir`: the first default name
/// found, then the first override file found. Empty when the folder has no
/// Compose file. Unlike Compose, it does not look in parent folders.
pub fn compose_files_in(dir: &Path) -> Vec<PathBuf> {
    let first = |names: &[&str]| {
        names
            .iter()
            .map(|name| dir.join(name))
            .find(|path| path.is_file())
    };
    let Some(main) = first(&COMPOSE_NAMES) else {
        return Vec::new();
    };
    std::iter::once(main)
        .chain(first(&OVERRIDE_NAMES))
        .collect()
}

/// The project name in the output of `docker compose config --format json`.
pub fn project_name_from_config(json: &str) -> Result<String, String> {
    let value: serde_json::Value = serde_json::from_str(json)
        .map_err(|error| format!("cannot read the output of docker compose config: {error}"))?;
    value
        .get("name")
        .and_then(serde_json::Value::as_str)
        .filter(|name| !name.is_empty())
        .map(String::from)
        .ok_or_else(|| "docker compose config gave no project name".into())
}

#[cfg(test)]
mod tests;
