use crate::HostStatus;

/// What the checks need to know about the operating system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Platform {
    pub macos: bool,
    pub windows: bool,
    /// An Apple silicon Mac.
    pub apple_silicon: bool,
}

impl Platform {
    /// The platform Captain was built for.
    pub fn current() -> Self {
        let macos = cfg!(target_os = "macos");
        Self {
            macos,
            windows: cfg!(windows),
            apple_silicon: macos && cfg!(target_arch = "aarch64"),
        }
    }
}

/// What a command-line tool reported.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolProbe {
    Missing,
    /// The tool exists but did not run or did not answer. The text says why.
    Broken(String),
    /// The tool ran and printed this version.
    Found(String),
}

/// Whether the engine answered a live request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineProbe {
    /// Captain is still connecting.
    Connecting,
    Answered {
        version: String,
        api_version: String,
    },
    /// The engine did not answer. The text says why.
    NoAnswer(String),
}

/// Facts about this computer that need commands or the file system to find.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MachineFacts {
    pub lima: ToolProbe,
    pub docker: ToolProbe,
    pub compose: ToolProbe,
    /// Free bytes on the disk that holds the home folder, if Captain could read it.
    pub free_disk: Option<u64>,
    /// Bytes in the `*.log` files of Captain Engine's instance folder. `None` when
    /// the folder does not exist.
    pub lima_logs: Option<u64>,
    /// Whether Rosetta is installed. `None` where the question does not apply.
    pub rosetta: Option<bool>,
}

/// Everything the checks read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Facts {
    pub platform: Platform,
    /// Captain Engine's status when the settings choose it, else `None`.
    pub captain_engine: Option<HostStatus>,
    pub engine: EngineProbe,
    pub machine: MachineFacts,
    /// The mistake in `settings.json` that keeps Captain on the last good
    /// settings, as the user sees it. `None` when the file reads cleanly.
    pub settings_problem: Option<String>,
}
