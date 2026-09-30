use std::path::{Path, PathBuf};

use crate::docker_context::config_dir;
use crate::tools::Bundle;

/// Linking tools and editing shell files works on macOS and Linux.
pub const SUPPORTED: bool = cfg!(unix);

/// What the card and the CLI say where [`SUPPORTED`] is false.
pub const UNSUPPORTED: &str = "Command-line tools are not supported on Windows yet.";

/// The places Captain changes for the command-line tools.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolPaths {
    pub home: PathBuf,
    /// `~/.captain/bin`, for `PATH`.
    pub bin: PathBuf,
    /// `~/.captain/cli-plugins`, for `cliPluginsExtraDirs`.
    pub plugins: PathBuf,
    /// The user's docker `config.json`.
    pub docker_config: PathBuf,
}

impl ToolPaths {
    /// The paths under `home`. `docker_config_dir` is `DOCKER_CONFIG`, if set.
    pub fn new(home: &Path, docker_config_dir: Option<PathBuf>) -> Self {
        let docker_dir =
            config_dir(docker_config_dir, Some(home)).unwrap_or_else(|| home.join(".docker"));
        Self {
            home: home.to_path_buf(),
            bin: home.join(".captain/bin"),
            plugins: home.join(".captain/cli-plugins"),
            docker_config: docker_dir.join("config.json"),
        }
    }

    /// The paths of this user, from `HOME` and `DOCKER_CONFIG`.
    pub fn user() -> Option<Self> {
        let home = std::env::home_dir()?;
        Some(Self::new(
            &home,
            std::env::var_os("DOCKER_CONFIG").map(PathBuf::from),
        ))
    }

    /// `path` with the home folder as `~`, for messages.
    pub fn tilde(&self, path: &Path) -> String {
        match path.strip_prefix(&self.home) {
            Ok(rest) => format!("~/{}", rest.display()),
            Err(_) => path.display().to_string(),
        }
    }
}

/// The `Captain.app` this process runs from. The CLI may run through the
/// `~/.captain/bin/captain` link, so the path is resolved first.
pub fn running_bundle() -> Option<Bundle> {
    let exe = std::env::current_exe().ok()?;
    let exe = exe.canonicalize().unwrap_or(exe);
    Bundle::from_exe(&exe)
}

/// The user's login shell: `SHELL`, else zsh on macOS.
pub fn login_shell() -> Option<PathBuf> {
    std::env::var_os("SHELL")
        .filter(|shell| !shell.is_empty())
        .map(PathBuf::from)
        .or_else(|| cfg!(target_os = "macos").then(|| PathBuf::from("/bin/zsh")))
}
