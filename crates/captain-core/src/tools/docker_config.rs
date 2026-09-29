//! The docker CLI finds plugins in the `cliPluginsExtraDirs` of its `config.json`
//! first, then in `<config dir>/cli-plugins`
//! (<https://github.com/docker/cli/blob/master/cli-plugins/manager/manager.go>).
//! There is no environment variable for it. To use the bundled Compose and Buildx
//! without editing `~/.docker/config.json`, Captain runs the CLI with its own
//! config folder (`DOCKER_CONFIG`), and this builds that folder's `config.json`.

use std::path::Path;

use serde_json::Value;

const EXTRA_DIRS: &str = "cliPluginsExtraDirs";

/// A copy of the user's `config.json` (`user`, if any) that keeps credentials,
/// credential helpers, and proxies, and puts `bundled_plugins` first in
/// `cliPluginsExtraDirs`, then the user's extra folders, then `user_plugins` (the
/// user's `cli-plugins` folder, which the CLI no longer sees on its own). The
/// `currentContext` goes, because Captain sets `DOCKER_HOST` and the CLI refuses
/// both together. A `user` that does not parse counts as empty.
pub fn docker_config(user: Option<&str>, bundled_plugins: &Path, user_plugins: &Path) -> String {
    let mut config = user
        .and_then(|text| serde_json::from_str::<Value>(text).ok())
        .and_then(|value| match value {
            Value::Object(map) => Some(map),
            _ => None,
        })
        .unwrap_or_default();
    config.remove("currentContext");

    let user_extra = match config.remove(EXTRA_DIRS) {
        Some(Value::Array(dirs)) => dirs,
        _ => Vec::new(),
    };
    let mut dirs = vec![path_value(bundled_plugins)];
    dirs.extend(user_extra);
    dirs.push(path_value(user_plugins));
    config.insert(EXTRA_DIRS.into(), Value::Array(dirs));

    let mut text = serde_json::to_string_pretty(&Value::Object(config)).unwrap_or_default();
    text.push('\n');
    text
}

fn path_value(path: &Path) -> Value {
    Value::String(path.display().to_string())
}

#[cfg(test)]
mod tests;
