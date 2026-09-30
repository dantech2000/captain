//! The AI agents Captain can connect to its MCP server, and how each one keeps its
//! list of servers. Sources for each client are in docs/features/0038-agent-tools.md.

/// An MCP client that Captain knows how to connect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AgentClient {
    ClaudeCode,
    Codex,
    GeminiCli,
    VsCode,
    Cursor,
    Zed,
    ClaudeDesktop,
}

/// How a client keeps its servers: a JSON file with the servers under one key, or
/// Codex's TOML.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFormat {
    /// JSON (comments allowed) with the servers in the object at this key.
    Json(&'static str),
    /// TOML with a `[mcp_servers.captain]` table.
    CodexToml,
}

/// The name of Captain's server in every client.
pub const SERVER_NAME: &str = "captain";

impl AgentClient {
    pub const ALL: [AgentClient; 7] = [
        AgentClient::ClaudeCode,
        AgentClient::Codex,
        AgentClient::GeminiCli,
        AgentClient::VsCode,
        AgentClient::Cursor,
        AgentClient::Zed,
        AgentClient::ClaudeDesktop,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::ClaudeCode => "Claude Code",
            Self::Codex => "Codex",
            Self::GeminiCli => "Gemini CLI",
            Self::VsCode => "VS Code",
            Self::Cursor => "Cursor",
            Self::Zed => "Zed",
            Self::ClaudeDesktop => "Claude Desktop",
        }
    }

    /// A short ID for element IDs, such as `claude-code`.
    pub fn id(self) -> &'static str {
        match self {
            Self::ClaudeCode => "claude-code",
            Self::Codex => "codex",
            Self::GeminiCli => "gemini-cli",
            Self::VsCode => "vs-code",
            Self::Cursor => "cursor",
            Self::Zed => "zed",
            Self::ClaudeDesktop => "claude-desktop",
        }
    }

    /// The command that finds the client, and that connects it where the client
    /// has an installer.
    pub fn command(self) -> Option<&'static str> {
        match self {
            Self::ClaudeCode => Some("claude"),
            Self::Codex => Some("codex"),
            Self::GeminiCli => Some("gemini"),
            Self::VsCode => Some("code"),
            Self::Cursor => Some("cursor"),
            Self::Zed => Some("zed"),
            Self::ClaudeDesktop => None,
        }
    }

    pub fn format(self) -> ConfigFormat {
        match self {
            Self::Codex => ConfigFormat::CodexToml,
            Self::VsCode => ConfigFormat::Json("servers"),
            Self::Zed => ConfigFormat::Json("context_servers"),
            _ => ConfigFormat::Json("mcpServers"),
        }
    }

    /// What the user must do after a change, if anything.
    pub fn after_change(self) -> Option<&'static str> {
        match self {
            Self::ClaudeDesktop => Some("Quit and open Claude Desktop again to load the change."),
            Self::ClaudeCode | Self::Codex | Self::GeminiCli => {
                Some("Sessions that are open see the change when they start again.")
            }
            _ => None,
        }
    }
}

/// A name for the client in an activity entry, from the `clientInfo.name` it sent,
/// such as `claude-code`. Unknown names stay as they are.
pub fn client_label(name: &str) -> String {
    let lower = name.to_ascii_lowercase();
    let known = [
        ("claude-code", "Claude Code"),
        ("claude-ai", "Claude Desktop"),
        ("codex", "Codex"),
        ("gemini", "Gemini CLI"),
        ("cursor", "Cursor"),
        ("visual studio code", "VS Code"),
        ("zed", "Zed"),
    ];
    known
        .iter()
        .find(|(fragment, _)| lower.contains(fragment))
        .map_or_else(|| name.to_string(), |(_, label)| label.to_string())
}
