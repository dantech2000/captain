//! Where each client keeps its MCP servers, under one home folder, so tests can use
//! a temporary one.

use std::path::{Path, PathBuf};

use super::client::AgentClient;

/// The folders the clients' files live in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientPaths {
    pub home: PathBuf,
    /// The OS's folder for app settings: `~/Library/Application Support` on macOS,
    /// `~/.config` (or `XDG_CONFIG_HOME`) on Linux, `%APPDATA%` on Windows.
    pub app_config: PathBuf,
    /// `CODEX_HOME`, or `~/.codex`.
    pub codex_home: PathBuf,
}

impl ClientPaths {
    /// The paths under `home` with the OS's default layout.
    pub fn new(home: &Path) -> Self {
        let app_config = if cfg!(target_os = "macos") {
            home.join("Library/Application Support")
        } else if cfg!(windows) {
            home.join("AppData/Roaming")
        } else {
            home.join(".config")
        };
        Self {
            home: home.to_path_buf(),
            app_config,
            codex_home: home.join(".codex"),
        }
    }

    /// This user's paths, with `XDG_CONFIG_HOME`, `APPDATA`, and `CODEX_HOME`.
    pub fn user() -> Option<Self> {
        let home = std::env::home_dir()?;
        let mut paths = Self::new(&home);
        let var = |name: &str| std::env::var_os(name).filter(|dir| !dir.is_empty());
        let app_config = match () {
            _ if cfg!(windows) => var("APPDATA"),
            _ if cfg!(target_os = "macos") => None,
            _ => var("XDG_CONFIG_HOME"),
        };
        if let Some(dir) = app_config {
            paths.app_config = PathBuf::from(dir);
        }
        if let Some(dir) = var("CODEX_HOME") {
            paths.codex_home = PathBuf::from(dir);
        }
        Some(paths)
    }

    /// The file that lists `client`'s user-wide MCP servers, or `None` where the
    /// client does not run (Claude Desktop on Linux).
    pub fn config_file(&self, client: AgentClient) -> Option<PathBuf> {
        Some(match client {
            AgentClient::ClaudeCode => self.home.join(".claude.json"),
            AgentClient::Codex => self.codex_home.join("config.toml"),
            AgentClient::GeminiCli => self.home.join(".gemini/settings.json"),
            AgentClient::VsCode => self.app_config.join("Code/User/mcp.json"),
            AgentClient::Cursor => self.home.join(".cursor/mcp.json"),
            AgentClient::Zed if cfg!(windows) => self.app_config.join("Zed/settings.json"),
            AgentClient::Zed => self.zed_dir().join("settings.json"),
            AgentClient::ClaudeDesktop if cfg!(any(target_os = "macos", windows)) => {
                self.app_config.join("Claude/claude_desktop_config.json")
            }
            AgentClient::ClaudeDesktop => return None,
        })
    }

    /// `~/.config/zed` on macOS and Linux; Zed uses it on both.
    fn zed_dir(&self) -> PathBuf {
        if cfg!(target_os = "macos") {
            self.home.join(".config/zed")
        } else {
            self.app_config.join("zed")
        }
    }

    /// `path` with the home folder as `~`, for messages.
    pub fn tilde(&self, path: &Path) -> String {
        match path.strip_prefix(&self.home) {
            Ok(rest) => format!("~/{}", rest.display()),
            Err(_) => path.display().to_string(),
        }
    }
}

/// The `captain` command that agents run, as an absolute path:
/// `~/.captain/bin/captain` when the command-line tools put it there, so it stays
/// right when Captain.app moves; else the copy inside Captain.app; else
/// `captain-cli` (`captain-cli.exe` on Windows) next to `exe`, in a development
/// build.
pub fn captain_command(home: &Path, exe: &Path) -> Option<PathBuf> {
    let linked = home.join(".captain/bin/captain");
    let bundled = crate::tools::Bundle::from_exe(exe).map(|bundle| bundle.captain_cli());
    let cli = format!("captain-cli{}", std::env::consts::EXE_SUFFIX);
    let built = exe.parent().map(|dir| dir.join(cli));
    [Some(linked), bundled, built]
        .into_iter()
        .flatten()
        .find(|path| path.exists())
}

#[cfg(test)]
mod tests;
