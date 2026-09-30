//! Captain's plugin folder in `cliPluginsExtraDirs` of the user's docker
//! `config.json`. The docker CLI searches those folders before
//! `~/.docker/cli-plugins` and takes the first match
//! (<https://github.com/docker/cli/blob/master/cli-plugins/manager/manager.go>).

use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::{Map, Value};

use crate::file_replace::{Mode, backup_once, commit, sibling, temp_path, write_new};
use crate::link_target::link_target;
use crate::process_lock::ProcessLock;

const EXTRA_DIRS: &str = "cliPluginsExtraDirs";

/// `text` (a `config.json`, `None` when there is none) with `dir` first in
/// `cliPluginsExtraDirs`. Other keys keep their order. `None` when `dir` is
/// already listed.
pub fn with_plugin_dir(text: Option<&str>, dir: &Path) -> Result<Option<String>, String> {
    let mut config = parse(text)?;
    let dir = dir_value(dir);
    let dirs = config
        .entry(EXTRA_DIRS)
        .or_insert_with(|| Value::Array(Vec::new()));
    let Value::Array(dirs) = dirs else {
        return Err(format!("{EXTRA_DIRS} is not a list."));
    };
    if dirs.contains(&dir) {
        return Ok(None);
    }
    dirs.insert(0, dir);
    Ok(Some(to_text(&config)))
}

/// `text` without `dir` in `cliPluginsExtraDirs`, and without the key when the
/// list becomes empty. `None` when `dir` is not listed.
pub fn without_plugin_dir(text: &str, dir: &Path) -> Result<Option<String>, String> {
    let mut config = parse(Some(text))?;
    let dir = dir_value(dir);
    let Some(Value::Array(dirs)) = config.get_mut(EXTRA_DIRS) else {
        return Ok(None);
    };
    if !dirs.contains(&dir) {
        return Ok(None);
    }
    dirs.retain(|entry| *entry != dir);
    if dirs.is_empty() {
        config.shift_remove(EXTRA_DIRS);
    }
    Ok(Some(to_text(&config)))
}

/// True if the file at `config` lists `dir` in `cliPluginsExtraDirs`.
pub fn has_plugin_dir(config: &Path, dir: &Path) -> bool {
    std::fs::read_to_string(config)
        .ok()
        .is_some_and(|text| with_plugin_dir(Some(&text), dir) == Ok(None))
}

/// Adds `dir` to the file at `config`. Returns true if the file changed.
pub fn add_plugin_dir(config: &Path, dir: &Path) -> Result<bool, String> {
    update(config, dir, |text| with_plugin_dir(text, dir), |_| {})
}

/// Removes `dir` from the file at `config`. Returns true if the file changed.
pub fn remove_plugin_dir(config: &Path, dir: &Path) -> Result<bool, String> {
    let change = |text: Option<&str>| text.map_or(Ok(None), |text| without_plugin_dir(text, dir));
    update(config, dir, change, |_| {})
}

/// How many times Captain reads and merges again when another program, such as
/// `docker login`, changed the file while Captain wrote it.
const ATTEMPTS: usize = 3;

/// The lock that Captain's own writers of `config.json` hold, the app and the
/// CLI alike: `docker-config.lock` next to the plugin folder, in `~/.captain`.
fn lock_path(plugins: &Path) -> PathBuf {
    plugins.with_file_name("docker-config.lock")
}

/// Writes `change` of the file at `link` to `link`'s target, through a new file
/// and a rename. Right before the rename, Captain reads the file again; if another
/// program changed it meanwhile, Captain merges again from the new text.
/// `before_commit` runs at that point, for tests.
///
/// The first write copies the old file to `config.json.captain-backup`, next to
/// the link. Only the owner can read the new file, because it can hold registry
/// logins.
fn update(
    link: &Path,
    plugins: &Path,
    change: impl Fn(Option<&str>) -> Result<Option<String>, String>,
    mut before_commit: impl FnMut(&Path),
) -> Result<bool, String> {
    let path = &link_target(link);
    let fail = |error: std::io::Error| format!("Cannot write {}: {error}", path.display());
    let lock = lock_path(plugins);
    let _lock = ProcessLock::acquire(&lock)
        .map_err(|error| format!("Cannot lock {}: {error}", lock.display()))?;
    for _ in 0..ATTEMPTS {
        let original = read(path)?;
        let text = original
            .clone()
            .map(String::from_utf8)
            .transpose()
            .map_err(|_| format!("{}: It is not UTF-8 text.", link.display()))?;
        let Some(new) =
            change(text.as_deref()).map_err(|error| format!("{}: {error}", link.display()))?
        else {
            return Ok(false);
        };
        backup_once(path, &sibling(link, "captain-backup")).map_err(fail)?;
        let temp = temp_path(path);
        write_new(&temp, path, new.as_bytes(), Mode::Private).map_err(fail)?;
        before_commit(path);
        if read(path)? != original {
            std::fs::remove_file(&temp).ok();
            continue;
        }
        commit(&temp, path).map_err(fail)?;
        return Ok(true);
    }
    Err(format!(
        "{} kept changing while Captain wrote it. Try again.",
        path.display()
    ))
}

fn parse(text: Option<&str>) -> Result<Map<String, Value>, String> {
    match text.map(str::trim).filter(|text| !text.is_empty()) {
        None => Ok(Map::new()),
        Some(text) => match serde_json::from_str(text) {
            Ok(Value::Object(map)) => Ok(map),
            Ok(_) => Err("It is not a JSON object.".into()),
            Err(error) => Err(format!("It is not valid JSON: {error}")),
        },
    }
}

fn dir_value(dir: &Path) -> Value {
    Value::String(dir.display().to_string())
}

/// The docker CLI writes `config.json` with tabs, so Captain does too.
fn to_text(config: &Map<String, Value>) -> String {
    let mut bytes = Vec::new();
    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"\t");
    let mut serializer = serde_json::Serializer::with_formatter(&mut bytes, formatter);
    // Writing a map of JSON values to a Vec cannot fail.
    let _ = config.serialize(&mut serializer);
    let mut text = String::from_utf8(bytes).unwrap_or_default();
    text.push('\n');
    text
}

fn read(config: &Path) -> Result<Option<Vec<u8>>, String> {
    match std::fs::read(config) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(format!("Cannot read {}: {error}", config.display())),
    }
}

#[cfg(test)]
mod tests;
