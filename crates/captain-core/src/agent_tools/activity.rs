//! The agent activity log, `~/.captain/agent-activity.jsonl`: one JSON line per tool
//! call that `captain mcp` answered. The server appends; the app reads it for the
//! status bar and the Agent activity list. The file stays under [`ACTIVITY_CAP`].

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use super::settings::AgentAction;
use crate::file_replace::{Mode, replace};

/// The file name in `~/.captain`.
pub const ACTIVITY_FILE: &str = "agent-activity.jsonl";
/// The most bytes the file keeps. Past it, the older half goes.
pub const ACTIVITY_CAP: u64 = 256 * 1024;
/// The longest result or argument text an entry keeps, in characters.
const MAX_TEXT: usize = 300;

/// One tool call.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Activity {
    /// Unix seconds.
    pub at: i64,
    /// The client's name from its MCP handshake, such as `claude-code`.
    pub client: String,
    pub tool: String,
    #[serde(default)]
    pub arguments: Map<String, Value>,
    /// False when the call failed or was refused.
    pub ok: bool,
    /// The first line of the answer, or why it failed.
    #[serde(default)]
    pub result: String,
}

impl Activity {
    /// An entry with long texts cut.
    pub fn new(
        at: i64,
        client: &str,
        tool: &str,
        arguments: Map<String, Value>,
        ok: bool,
        result: &str,
    ) -> Self {
        let arguments = arguments
            .into_iter()
            .map(|(key, value)| match value {
                Value::String(text) => (key, Value::String(cut(&text))),
                other => (key, other),
            })
            .collect();
        Self {
            at,
            client: cut(client),
            tool: cut(tool),
            arguments,
            ok,
            result: cut(result.lines().next().unwrap_or_default()),
        }
    }

    /// True for `start`, `stop`, `restart`, `run_task`, and `raise_memory`.
    pub fn is_action(&self) -> bool {
        AgentAction::of_tool(&self.tool).is_some()
    }

    /// What the call was about: the container, project, and task arguments, such as
    /// `shop / migrate`.
    pub fn target(&self) -> Option<String> {
        let parts: Vec<&str> = ["container", "project", "task"]
            .iter()
            .filter_map(|key| self.arguments.get(*key)?.as_str())
            .collect();
        (!parts.is_empty()).then(|| parts.join(" / "))
    }

    /// For example "restart shop-api-1", or "list_projects".
    pub fn summary(&self) -> String {
        match self.target() {
            Some(target) => format!("{} {target}", self.tool),
            None => self.tool.clone(),
        }
    }
}

/// `~/.captain/agent-activity.jsonl` under `home`.
pub fn activity_path(home: &Path) -> PathBuf {
    home.join(".captain").join(ACTIVITY_FILE)
}

/// Adds `entry` as one line, and drops the older half once the file passes
/// [`ACTIVITY_CAP`].
pub fn append_activity(path: &Path, entry: &Activity) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let mut line = serde_json::to_string(entry).map_err(io::Error::other)?;
    line.push('\n');
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    // One write per line, so lines from two servers do not mix.
    file.write_all(line.as_bytes())?;
    if file.metadata()?.len() > ACTIVITY_CAP {
        let text = fs::read_to_string(path)?;
        replace(path, newer_half(&text).as_bytes(), Mode::Keep)?;
    }
    Ok(())
}

/// The newest `limit` entries, newest first. Lines that do not parse are skipped;
/// a missing file has none.
pub fn read_activity(path: &Path, limit: usize) -> Vec<Activity> {
    let Ok(text) = fs::read_to_string(path) else {
        return Vec::new();
    };
    text.lines()
        .rev()
        .filter_map(|line| serde_json::from_str(line).ok())
        .take(limit)
        .collect()
}

/// The whole lines of the newer half of `text`.
fn newer_half(text: &str) -> &str {
    let middle = text.len() / 2;
    match text[middle..].find('\n') {
        Some(end) => &text[middle + end + 1..],
        None => "",
    }
}

fn cut(text: &str) -> String {
    match text.char_indices().nth(MAX_TEXT) {
        Some((end, _)) => format!("{}\u{2026}", &text[..end]),
        None => text.to_string(),
    }
}

#[cfg(test)]
mod tests;
