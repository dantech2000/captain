//! Reads the Docker CLI context store. Each context lives in
//! `<config dir>/contexts/meta/<sha256 of the name>/meta.json`, and `config.json` names
//! the default in `currentContext`. See docs/features/0026-contexts-and-remote-hosts.md.

use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use sha2::{Digest, Sha256};

/// The context Captain creates for Captain Engine.
pub const CAPTAIN_CONTEXT: &str = "captain";

/// The built-in context. It has no metadata; it means `DOCKER_HOST` or the default socket.
const DEFAULT_CONTEXT: &str = "default";

/// A context from the store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DockerContext {
    pub name: String,
    pub description: String,
    /// The Docker endpoint, for example `unix:///var/run/docker.sock`. `None` for a
    /// context with no Docker endpoint.
    pub host: Option<String>,
}

/// The stored contexts, sorted by name, and the one `config.json` makes the default.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ContextList {
    pub contexts: Vec<DockerContext>,
    pub current: Option<String>,
}

impl ContextList {
    pub fn get(&self, name: &str) -> Option<&DockerContext> {
        self.contexts.iter().find(|context| context.name == name)
    }

    pub fn is_current(&self, name: &str) -> bool {
        self.current.as_deref() == Some(name)
    }
}

/// The Docker CLI config dir: `DOCKER_CONFIG` when set, else `~/.docker`.
pub fn config_dir(docker_config: Option<PathBuf>, home: Option<&Path>) -> Option<PathBuf> {
    docker_config
        .filter(|dir| !dir.as_os_str().is_empty())
        .or_else(|| home.map(|home| home.join(".docker")))
}

/// The directory name of a context: the SHA-256 hex digest of its name.
pub fn meta_dir_name(name: &str) -> String {
    Sha256::digest(name.as_bytes())
        .iter()
        .fold(String::with_capacity(64), |mut hex, byte| {
            let _ = write!(hex, "{byte:02x}");
            hex
        })
}

/// Parses a `meta.json` file. `None` if it has no name.
pub fn parse_meta(bytes: &[u8]) -> Option<DockerContext> {
    let meta: Value = serde_json::from_slice(bytes).ok()?;
    let text = |pointer| meta.pointer(pointer).and_then(Value::as_str);
    Some(DockerContext {
        name: text("/Name").filter(|name| !name.is_empty())?.to_string(),
        description: text("/Metadata/Description")
            .unwrap_or_default()
            .to_string(),
        host: text("/Endpoints/docker/Host").map(ToString::to_string),
    })
}

/// The `currentContext` of a `config.json` file, unless it is empty or `default`.
pub fn parse_current(config: &[u8]) -> Option<String> {
    let config: Value = serde_json::from_slice(config).ok()?;
    let name = config.get("currentContext")?.as_str()?;
    (!name.is_empty() && name != DEFAULT_CONTEXT).then(|| name.to_string())
}

/// Reads every context in `docker_dir`. Files that do not parse are left out.
pub fn read_contexts(docker_dir: &Path) -> ContextList {
    let meta_dir = docker_dir.join("contexts").join("meta");
    let mut contexts: Vec<DockerContext> = fs::read_dir(meta_dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| parse_meta(&fs::read(entry.path().join("meta.json")).ok()?))
        .collect();
    contexts.sort_by(|a, b| a.name.cmp(&b.name));
    let current = fs::read(docker_dir.join("config.json"))
        .ok()
        .and_then(|config| parse_current(&config));
    ContextList { contexts, current }
}

/// The Docker host of the context named `name`, read from its hash directory.
pub fn context_host(docker_dir: &Path, name: &str) -> Option<String> {
    let path = docker_dir
        .join("contexts")
        .join("meta")
        .join(meta_dir_name(name))
        .join("meta.json");
    parse_meta(&fs::read(path).ok()?)
        .filter(|context| context.name == name)?
        .host
}

/// The Docker host of the current context. `context_override` is `DOCKER_CONTEXT`,
/// which wins over `config.json`. The `default` context gives `None`.
pub fn current_host(docker_dir: &Path, context_override: Option<&str>) -> Option<String> {
    let name = match context_override {
        Some(name) => name.to_string(),
        None => parse_current(&fs::read(docker_dir.join("config.json")).ok()?)?,
    };
    if name.is_empty() || name == DEFAULT_CONTEXT {
        return None;
    }
    context_host(docker_dir, &name)
}

#[cfg(test)]
mod tests;
