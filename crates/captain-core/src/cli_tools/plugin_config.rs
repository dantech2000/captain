//! Captain's plugin folder in `cliPluginsExtraDirs` of the user's docker
//! `config.json`. The docker CLI searches those folders before
//! `~/.docker/cli-plugins` and takes the first match
//! (<https://github.com/docker/cli/blob/master/cli-plugins/manager/manager.go>).

use std::ffi::OsStr;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::{Map, Value};

use crate::link_target::link_target;

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
    let text = read(config)?;
    let changed = with_plugin_dir(text.as_deref(), dir)
        .map_err(|error| format!("{}: {error}", config.display()))?;
    changed
        .as_deref()
        .map(|text| write(config, text))
        .transpose()?;
    Ok(changed.is_some())
}

/// Removes `dir` from the file at `config`. Returns true if the file changed.
pub fn remove_plugin_dir(config: &Path, dir: &Path) -> Result<bool, String> {
    let Some(text) = read(config)? else {
        return Ok(false);
    };
    let changed =
        without_plugin_dir(&text, dir).map_err(|error| format!("{}: {error}", config.display()))?;
    changed
        .as_deref()
        .map(|text| write(config, text))
        .transpose()?;
    Ok(changed.is_some())
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

fn read(config: &Path) -> Result<Option<String>, String> {
    match std::fs::read_to_string(config) {
        Ok(text) => Ok(Some(text)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(format!("Cannot read {}: {error}", config.display())),
    }
}

/// Writes `text` to `link`'s target through a new file and a rename. The first
/// write copies the old file to `config.json.captain-backup`, next to the link.
/// Only the owner can read the new file, because it can hold registry logins.
fn write(link: &Path, text: &str) -> Result<(), String> {
    let path = &link_target(link);
    let fail = |error: std::io::Error| format!("Cannot write {}: {error}", path.display());
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(fail)?;
    }
    let backup = sibling(link, "captain-backup");
    if path.exists() && !backup.exists() {
        std::fs::copy(path, &backup).map_err(fail)?;
    }
    let temp = sibling(path, "captain-new");
    std::fs::remove_file(&temp).ok();
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    let written = options
        .open(&temp)
        .and_then(|mut file| file.write_all(text.as_bytes()))
        .and_then(|()| std::fs::rename(&temp, path));
    if written.is_err() {
        std::fs::remove_file(&temp).ok();
    }
    written.map_err(fail)
}

/// `config.json` becomes `config.json.<suffix>` in the same folder.
fn sibling(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path
        .file_name()
        .unwrap_or(OsStr::new("config.json"))
        .to_os_string();
    name.push(format!(".{suffix}"));
    path.with_file_name(name)
}

#[cfg(test)]
mod tests;
