//! The `agent_tools` settings group: whether `captain mcp` answers agents at all,
//! and which actions they may run. The server reads it on every call, so a change
//! in Settings applies at once. See docs/features/0038-agent-tools.md.

use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};

/// An action that agents may run through Captain's MCP server when the user allows
/// it. Each one is also the name of its tool.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum AgentAction {
    /// Start a container or a Compose project.
    Start,
    /// Stop a container or a Compose project.
    Stop,
    /// Restart a container or a Compose project.
    Restart,
    /// Run a task that a Compose file declares in `x-captain.tasks`.
    RunTask,
    /// Raise the memory limit of a container: twice the old one, at least 512 MB.
    RaiseMemory,
}

impl AgentAction {
    pub const ALL: [AgentAction; 5] = [
        AgentAction::Start,
        AgentAction::Stop,
        AgentAction::Restart,
        AgentAction::RunTask,
        AgentAction::RaiseMemory,
    ];

    /// The tool name and the value in `agent_tools.actions`, such as `run_task`.
    pub fn name(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Stop => "stop",
            Self::Restart => "restart",
            Self::RunTask => "run_task",
            Self::RaiseMemory => "raise_memory",
        }
    }

    /// The checkbox label, such as "Run tasks".
    pub fn label(self) -> &'static str {
        match self {
            Self::Start => "Start",
            Self::Stop => "Stop",
            Self::Restart => "Restart",
            Self::RunTask => "Run tasks",
            Self::RaiseMemory => "Raise memory",
        }
    }

    /// The action whose tool is `tool`, if it is one.
    pub fn of_tool(tool: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|action| action.name() == tool)
    }
}

/// The saved choices for AI agents.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct AgentToolsSettings {
    /// Let AI agents connected to `captain mcp` read Captain's engine, projects,
    /// containers, problems, logs, and disk use. While it is off, the server lists
    /// only `help`. Applies at once.
    #[schemars(example = true)]
    pub enabled: bool,
    /// The actions agents may run: `start`, `stop`, `restart`, `run_task` (only the
    /// tasks in a Compose file's `x-captain.tasks`), and `raise_memory`. Each one is
    /// a tool that agents see only while it is here. Applies at once.
    #[serde(deserialize_with = "known_actions")]
    #[schemars(example = vec![AgentAction::Restart, AgentAction::RunTask])]
    pub actions: Vec<AgentAction>,
}

impl AgentToolsSettings {
    /// True if agents may run `action`.
    pub fn allows(&self, action: AgentAction) -> bool {
        self.enabled && self.actions.contains(&action)
    }

    /// Allows or disallows `action`, keeping the list in [`AgentAction::ALL`] order.
    pub fn set_allowed(&mut self, action: AgentAction, allowed: bool) {
        self.actions.retain(|other| *other != action);
        if allowed {
            self.actions.push(action);
            self.actions.sort();
        }
    }

    /// Why agents may not call `tool` now, naming the setting to change, or `Ok`.
    /// `help` always answers.
    pub fn gate(&self, tool: &str) -> Result<(), String> {
        if tool == "help" {
            return Ok(());
        }
        if !self.enabled {
            return Err(
                "Captain's agent tools are off. The user can turn them on in \
                 Captain under Settings > AI agents (the agent_tools.enabled setting)."
                    .into(),
            );
        }
        match AgentAction::of_tool(tool) {
            Some(action) if !self.actions.contains(&action) => Err(format!(
                "The {} action is off. The user can allow it in Captain under Settings > AI \
                 agents (add \"{}\" to the agent_tools.actions setting).",
                action.name(),
                action.name()
            )),
            _ => Ok(()),
        }
    }
}

/// Reads the list and skips values this build does not know, so one new action
/// from a later version does not turn off the others. The file check still names
/// the unknown value.
fn known_actions<'de, D>(deserializer: D) -> Result<Vec<AgentAction>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    Ok(value
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|item| AgentAction::deserialize(item).ok())
        .collect())
}

#[cfg(test)]
mod tests;
