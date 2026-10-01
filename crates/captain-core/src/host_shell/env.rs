use std::ffi::{OsStr, OsString};
use std::path::Path;

/// What the environment of a new shell depends on.
#[derive(Debug, Clone, Copy, Default)]
pub struct ShellEnvInput<'a> {
    /// The engine Captain is connected to, as a `DOCKER_HOST` URL.
    pub docker_host: Option<&'a str>,
    /// Captain's tool folder, for the front of `PATH`.
    pub tool_bin: Option<&'a Path>,
    /// Captain's own `PATH`.
    pub path: Option<&'a OsStr>,
    /// The `DOCKER_CONFIG` folder that finds Captain's Compose and Buildx.
    pub docker_config: Option<&'a Path>,
}

/// The changes to Captain's environment for a shell.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ShellEnv {
    pub set: Vec<(String, OsString)>,
    pub remove: Vec<String>,
}

impl ShellEnv {
    /// The value this environment sets for `key`.
    pub fn get(&self, key: &str) -> Option<&OsStr> {
        self.set
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value.as_os_str())
    }
}

/// The terminal variables, and `DOCKER_HOST` for the connected engine with
/// `DOCKER_CONTEXT` removed, because the docker CLI prefers `DOCKER_CONTEXT`
/// (<https://docs.docker.com/reference/cli/docker/#environment-variables>).
/// Captain's tool folder goes first on `PATH`, once. `KUBECONFIG` stays as it is.
pub fn shell_env(input: &ShellEnvInput) -> ShellEnv {
    let mut env = ShellEnv::default();
    let mut set = |key: &str, value: OsString| env.set.push((key.into(), value));
    set("TERM", "xterm-256color".into());
    set("COLORTERM", "truecolor".into());
    set("TERM_PROGRAM", "Captain".into());
    if let Some(host) = input.docker_host {
        set("DOCKER_HOST", host.into());
    }
    if let Some(bin) = input.tool_bin {
        let rest = input.path.map(std::env::split_paths).into_iter().flatten();
        let dirs = std::iter::once(bin.to_path_buf()).chain(rest.filter(|dir| dir != bin));
        if let Ok(path) = std::env::join_paths(dirs) {
            set("PATH", path);
        }
    }
    if let Some(dir) = input.docker_config {
        set("DOCKER_CONFIG", dir.into());
    }
    if input.docker_host.is_some() {
        env.remove.push("DOCKER_CONTEXT".into());
    }
    env
}

#[cfg(test)]
mod tests;
