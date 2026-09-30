/// One line of container output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogLine {
    pub stream: LogStream,
    pub text: String,
    pub level: LogLevel,
    /// When the engine received the line, in Unix seconds. `None` when the engine
    /// sent no time.
    pub timestamp: Option<i64>,
    /// The fraction of the second of `timestamp`, in nanoseconds.
    pub nanos: u32,
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
            timestamp: None,
            nanos: 0,
        }
    }

    /// Reads a line that Docker sent with timestamps: an RFC 3339 time, a space, then
    /// the text. A line that does not start with a time keeps all of its text.
    pub fn with_docker_time(stream: LogStream, raw: &str) -> Self {
        match timestamp::split(raw) {
            Some((time, text)) => Self {
                timestamp: Some(time),
                nanos: timestamp::nanos(raw),
                ..Self::new(stream, text)
            },
            None => Self::new(stream, raw),
        }
    }

    /// The time with full precision, as Unix seconds and nanoseconds.
    pub fn precise_time(&self) -> Option<(i64, u32)> {
        self.timestamp.map(|time| (time, self.nanos))
    }

    /// The time as `HH:MM:SS` in a zone `offset` seconds east of UTC.
    pub fn clock(&self, offset: i32) -> Option<String> {
        self.timestamp.map(|time| timestamp::clock(time, offset))
    }

    /// The line as plain text for the clipboard. With a zone offset, the clock goes
    /// in front of the text.
    pub fn copy_text(&self, offset: Option<i32>) -> String {
        match offset.and_then(|offset| self.clock(offset)) {
            Some(clock) => format!("{clock} {}", self.text),
            None => self.text.clone(),
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

mod timestamp;

pub use timestamp::parse as parse_rfc3339;

#[cfg(test)]
mod tests;
