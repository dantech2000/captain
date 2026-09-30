/// The result of one check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckState {
    Passed,
    Warning,
    Failed,
    /// The check does not apply to this computer or this engine.
    NotApplicable,
}

impl CheckState {
    pub fn label(self) -> &'static str {
        match self {
            Self::Passed => "Passed",
            Self::Warning => "Warning",
            Self::Failed => "Failed",
            Self::NotApplicable => "Not applicable",
        }
    }
}

/// Which check a [`Check`] reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CheckId {
    Engine,
    Lima,
    DockerCli,
    Compose,
    DiskSpace,
    LimaLogs,
    Rosetta,
    SettingsFile,
}

impl CheckId {
    pub fn title(self) -> &'static str {
        match self {
            Self::Engine => "The engine answers",
            Self::Lima => "Lima",
            Self::DockerCli => "Docker CLI",
            Self::Compose => "Docker Compose",
            Self::DiskSpace => "Free disk space",
            Self::LimaLogs => "Lima log size",
            Self::Rosetta => "Rosetta",
            Self::SettingsFile => "Settings file",
        }
    }
}

/// An action that fixes a failed check, or helps the user fix it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fix {
    StartEngine,
    /// Stops Captain Engine and starts it again.
    RestartEngine,
    /// Opens Captain Engine's instance folder in the file manager.
    ShowEngineFiles,
    /// Copies a shell command that the user runs in a terminal.
    CopyCommand(&'static str),
    /// Opens `settings.json` in the default text editor.
    OpenSettingsFile,
}

impl Fix {
    /// The button label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::StartEngine => "Start Captain Engine",
            Self::RestartEngine => "Restart Captain Engine",
            Self::ShowEngineFiles => "Show engine files",
            Self::CopyCommand(_) => "Copy command",
            Self::OpenSettingsFile => "Open settings.json",
        }
    }
}

/// One line on the Diagnostics page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Check {
    pub id: CheckId,
    pub state: CheckState,
    /// One line for the user.
    pub detail: String,
    pub fix: Option<Fix>,
}

impl Check {
    pub fn new(id: CheckId, state: CheckState, detail: impl Into<String>) -> Self {
        Self {
            id,
            state,
            detail: detail.into(),
            fix: None,
        }
    }

    pub fn with_fix(mut self, fix: Fix) -> Self {
        self.fix = Some(fix);
        self
    }
}
