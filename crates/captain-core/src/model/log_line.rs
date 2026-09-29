/// One line of container output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogLine {
    pub stream: LogStream,
    pub text: String,
    pub level: LogLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogStream {
    Stdout,
    Stderr,
}

/// A best guess at the severity of a line. Containers do not report it, so Captain
/// reads it from the text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
}

impl LogLine {
    pub fn new(stream: LogStream, text: impl Into<String>) -> Self {
        let text = text.into().trim_end_matches(['\r', '\n']).to_string();
        let level = LogLevel::detect(&text);
        Self {
            stream,
            text,
            level,
        }
    }
}

impl LogLevel {
    /// Looks for common level words. Stderr alone is not an error, because many
    /// programs write normal output there.
    pub fn detect(text: &str) -> Self {
        let lower = text.to_ascii_lowercase();
        let has = |words: &[&str]| words.iter().any(|word| lower.contains(word));
        if has(&["error", "fatal", "panic", "exception", "[err", " err "]) {
            Self::Error
        } else if has(&["warn", "[wrn"]) {
            Self::Warn
        } else {
            Self::Info
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
        }
    }
}

#[cfg(test)]
mod tests;
