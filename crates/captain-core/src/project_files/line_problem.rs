/// How bad a [`LineProblem`] is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// Compose or the build rejects the file.
    Error,
    /// The file works, but a check advises a change.
    Warning,
}

/// One message from a check, on a line of the edited file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineProblem {
    /// The 0-based line, or `None` when the message names no place in this file,
    /// for example an error in another Compose file of the project.
    pub line: Option<usize>,
    pub message: String,
    pub severity: Severity,
}

impl LineProblem {
    pub fn error(line: Option<usize>, message: impl Into<String>) -> Self {
        Self {
            line,
            message: message.into(),
            severity: Severity::Error,
        }
    }

    pub fn warning(line: Option<usize>, message: impl Into<String>) -> Self {
        Self {
            line,
            message: message.into(),
            severity: Severity::Warning,
        }
    }
}
