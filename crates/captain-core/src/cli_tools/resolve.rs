//! What a new terminal runs for each tool: `command -v` in the user's login shell.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use super::command::output_within;

/// The tools the Settings card and `captain tools status` show. `kubectl` and
/// `helm` are not Captain's, but Rancher Desktop links them too.
pub const SHOWN_TOOLS: [&str; 6] = [
    "docker",
    "docker-compose",
    "docker-credential-osxkeychain",
    "captain",
    "kubectl",
    "helm",
];

/// A login shell reads all its files, which can take a while.
const TIMEOUT: Duration = Duration::from_secs(10);

/// Where a tool comes from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolSource {
    Captain,
    RancherDesktop,
    DockerDesktop,
    Homebrew,
    Nix,
    Other(PathBuf),
    NotFound,
}

impl ToolSource {
    pub fn label(&self) -> String {
        match self {
            ToolSource::Captain => "Captain".into(),
            ToolSource::RancherDesktop => "Rancher Desktop (~/.rd/bin)".into(),
            ToolSource::DockerDesktop => "Docker Desktop".into(),
            ToolSource::Homebrew => "Homebrew".into(),
            ToolSource::Nix => "Nix".into(),
            ToolSource::Other(path) => path.display().to_string(),
            ToolSource::NotFound => "Not found".into(),
        }
    }
}

impl ToolSource {
    /// A short name for a sentence, such as "Rancher Desktop".
    pub fn name(&self) -> String {
        match self {
            ToolSource::RancherDesktop => "Rancher Desktop".into(),
            other => other.label(),
        }
    }
}

/// Names the source of the tool at `path`, which links to `resolved` in the end.
pub fn classify(path: &Path, resolved: &Path, home: &Path) -> ToolSource {
    let either = |test: &dyn Fn(&Path) -> bool| test(path) || test(resolved);
    let contains = |needle: &'static str| move |p: &Path| p.to_string_lossy().contains(needle);
    if either(&|p| p.starts_with(home.join(".captain"))) || either(&contains("/Captain.app/")) {
        ToolSource::Captain
    } else if either(&|p| p.starts_with(home.join(".rd")))
        || either(&contains("/Rancher Desktop.app/"))
    {
        ToolSource::RancherDesktop
    } else if either(&|p| p.starts_with(home.join(".docker/bin")))
        || either(&contains("/Docker.app/"))
    {
        ToolSource::DockerDesktop
    } else if either(&|p| {
        p.starts_with("/opt/homebrew")
            || p.starts_with("/usr/local/Cellar")
            || p.starts_with("/home/linuxbrew")
    }) {
        ToolSource::Homebrew
    } else if either(&|p| {
        p.starts_with("/nix")
            || p.starts_with("/etc/profiles/per-user")
            || p.starts_with("/run/current-system")
            || p.starts_with(home.join(".nix-profile"))
    }) {
        ToolSource::Nix
    } else {
        ToolSource::Other(path.to_path_buf())
    }
}

/// The paths in `command -v` output, by tool. A line counts when it is an absolute
/// path whose file name is a tool; other output of the shell files is ignored.
pub fn parse_command_v(output: &str, tools: &[&str]) -> Vec<(String, Option<PathBuf>)> {
    let paths: Vec<PathBuf> = output
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with('/'))
        .map(PathBuf::from)
        .collect();
    tools
        .iter()
        .map(|tool| {
            let found = paths
                .iter()
                .find(|path| path.file_name().is_some_and(|name| name == *tool));
            (tool.to_string(), found.cloned())
        })
        .collect()
}

/// Runs `<shell> -lic 'command -v <tools>'`, as a new terminal would, and names
/// where each tool comes from.
pub fn resolve_in_login_shell(
    shell: &Path,
    tools: &[&str],
    home: &Path,
) -> Result<Vec<(String, ToolSource)>, String> {
    let mut command = Command::new(shell);
    command
        .arg("-lic")
        .arg(format!("command -v {}", tools.join(" ")))
        .current_dir(home);
    let output = output_within(command, TIMEOUT)
        .map_err(|error| format!("Cannot run {}: {error}", shell.display()))?;
    let text = String::from_utf8_lossy(&output.stdout);
    Ok(parse_command_v(&text, tools)
        .into_iter()
        .map(|(tool, path)| {
            let source = match path {
                Some(path) => {
                    let resolved = path.canonicalize().unwrap_or_else(|_| path.clone());
                    classify(&path, &resolved, home)
                }
                None => ToolSource::NotFound,
            };
            (tool, source)
        })
        .collect())
}

#[cfg(test)]
mod tests;
