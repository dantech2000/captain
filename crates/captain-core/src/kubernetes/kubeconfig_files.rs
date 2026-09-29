//! The user's kubeconfig files: which ones kubectl reads, which one Captain writes,
//! and a backup before each write. See ADR 0010.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use serde_json::Value;

use super::kubeconfig::{self, CONTEXT};

/// The files kubectl reads: each entry of `KUBECONFIG`, or `~/.kube/config`.
pub fn kubeconfig_paths(env: Option<&OsStr>, home: &Path) -> Vec<PathBuf> {
    let paths: Vec<PathBuf> = env
        .map(|value| {
            std::env::split_paths(value)
                .filter(|path| !path.as_os_str().is_empty())
                .collect()
        })
        .unwrap_or_default();
    if paths.is_empty() {
        vec![home.join(".kube").join("config")]
    } else {
        paths
    }
}

/// [`kubeconfig_paths`] for this process.
pub fn user_kubeconfig_paths() -> Vec<PathBuf> {
    let home = std::env::home_dir().unwrap_or_default();
    kubeconfig_paths(std::env::var_os("KUBECONFIG").as_deref(), &home)
}

/// Reads a kubeconfig. A missing file is an empty config.
pub fn read_config(path: &Path) -> Result<Value, String> {
    match std::fs::read_to_string(path) {
        Ok(text) => {
            kubeconfig::parse(&text).map_err(|error| format!("{}: {error}", path.display()))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => kubeconfig::parse(""),
        Err(error) => Err(format!("Cannot read {}: {error}", path.display())),
    }
}

/// Writes `config` to `path`. An existing file is copied to `<name>.captain-backup`
/// first. The new file is written next to it and renamed, and only its owner can
/// read it, like the files kubectl writes.
pub fn write_config(path: &Path, config: &Value) -> Result<(), String> {
    let fail = |error: std::io::Error| format!("Cannot write {}: {error}", path.display());
    let yaml = kubeconfig::to_yaml(config)?;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(fail)?;
    }
    if path.exists() {
        std::fs::copy(path, sibling(path, "captain-backup")).map_err(fail)?;
    }
    let temp = sibling(path, "captain-new");
    std::fs::write(&temp, yaml).map_err(fail)?;
    restrict(&temp).map_err(fail)?;
    std::fs::rename(&temp, path).map_err(fail)
}

/// Merges the `captain` entries into the first file that has a `captain` context,
/// or else the first file, as `kubectl config` does. Returns the file.
pub fn install_captain(paths: &[PathBuf], captain: &Value) -> Result<PathBuf, String> {
    let target = first_where(paths, |config| {
        kubeconfig::contexts(config)
            .iter()
            .any(|name| name == CONTEXT)
    })?;
    let merged = kubeconfig::merge(&read_config(&target)?, captain);
    write_config(&target, &merged)?;
    Ok(target)
}

/// Removes the `captain` entries from each file that has them.
pub fn uninstall_captain(paths: &[PathBuf]) -> Result<(), String> {
    for path in paths {
        let config = read_config(path)?;
        let removed = kubeconfig::remove_captain(&config);
        if removed != config {
            write_config(path, &removed)?;
        }
    }
    Ok(())
}

/// The contexts of all files, and the current one, as kubectl merges them.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct KubeContexts {
    pub names: Vec<String>,
    pub current: Option<String>,
}

/// Reads the contexts. A file that cannot be read is skipped.
pub fn load_contexts(paths: &[PathBuf]) -> KubeContexts {
    let mut contexts = KubeContexts::default();
    for config in paths.iter().filter_map(|path| read_config(path).ok()) {
        for name in kubeconfig::contexts(&config) {
            if !contexts.names.contains(&name) {
                contexts.names.push(name);
            }
        }
        if contexts.current.is_none() {
            contexts.current = kubeconfig::current_context(&config);
        }
    }
    contexts
}

/// Makes `name` the current context in the first file that sets one, or else the
/// first file, as `kubectl config use-context` does.
pub fn use_context(paths: &[PathBuf], name: &str) -> Result<(), String> {
    let target = first_where(paths, |config| {
        kubeconfig::current_context(config).is_some()
    })?;
    let config = read_config(&target)?;
    write_config(&target, &kubeconfig::set_current(&config, name))
}

/// The first file whose config passes `test`, or the first file.
fn first_where(paths: &[PathBuf], test: impl Fn(&Value) -> bool) -> Result<PathBuf, String> {
    let first = paths.first().ok_or("There is no kubeconfig file.")?;
    let found = paths
        .iter()
        .find(|path| read_config(path).is_ok_and(|config| test(&config)));
    Ok(found.unwrap_or(first).clone())
}

/// `config` becomes `config.<suffix>` in the same folder.
fn sibling(path: &Path, suffix: &str) -> PathBuf {
    let name = path.file_name().unwrap_or(OsStr::new("config"));
    let mut name = name.to_os_string();
    name.push(format!(".{suffix}"));
    path.with_file_name(name)
}

#[cfg(unix)]
fn restrict(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
}

#[cfg(not(unix))]
fn restrict(_: &Path) -> std::io::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests;
