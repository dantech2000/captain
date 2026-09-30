//! Which log lines an agent asked for, and the limits on what comes back. The
//! limits keep one answer under Claude Code's 10,000-token warning.

use crate::model::{LogLevel, LogLine, parse_rfc3339};

/// Lines per container without a filter.
pub const DEFAULT_TAIL: usize = 100;
/// Lines per container with `errors_only` or `grep`, so a filter has lines to find.
pub const FILTERED_TAIL: usize = 1000;
/// The most lines an agent can ask for per container.
pub const MAX_TAIL: usize = 5000;
/// The most lines one answer holds.
pub const MAX_LINES: usize = 500;
/// The most bytes of lines one answer holds.
pub const MAX_BYTES: usize = 32 * 1024;
/// Longer lines are cut to this many characters.
pub const MAX_LINE_CHARS: usize = 2000;

/// What the `logs` tool reads and keeps.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LogQuery {
    /// Past lines per container, before the filters.
    pub tail: Option<usize>,
    /// Only lines from this Unix time on.
    pub since: Option<i64>,
    /// Only lines that look like errors.
    pub errors_only: bool,
    /// Only lines that contain this text, ignoring case.
    pub grep: Option<String>,
}

impl LogQuery {
    /// The lines to read per container: the asked tail, up to [`MAX_TAIL`], or a
    /// default that is larger when a filter drops lines.
    pub fn fetch_tail(&self) -> usize {
        let filtered = self.errors_only || self.grep.is_some();
        let default = if filtered {
            FILTERED_TAIL
        } else {
            DEFAULT_TAIL
        };
        self.tail.unwrap_or(default).clamp(1, MAX_TAIL)
    }

    /// True if `line` passes the filters.
    pub fn keeps(&self, line: &LogLine) -> bool {
        let level = !self.errors_only || line.level == LogLevel::Error;
        let text = self
            .grep
            .as_deref()
            .is_none_or(|grep| line.text.to_lowercase().contains(&grep.to_lowercase()));
        level && text
    }
}

/// Reads a `since` value as Unix seconds: an age such as `90s`, `10m`, `2h`, or
/// `1d` before `now`, a Unix time, or an RFC 3339 time.
pub fn parse_since(value: &str, now: i64) -> Result<i64, String> {
    let value = value.trim();
    let unit = |suffix: char| match suffix {
        's' => Some(1),
        'm' => Some(60),
        'h' => Some(3600),
        'd' => Some(86_400),
        _ => None,
    };
    if let Some(last) = value.chars().last()
        && let Some(seconds) = unit(last)
        && let Ok(count) = value[..value.len() - 1].parse::<i64>()
        && count >= 0
    {
        return Ok(now - count.saturating_mul(seconds));
    }
    if let Ok(time) = value.parse::<i64>()
        && time >= 0
    {
        return Ok(time);
    }
    parse_rfc3339(value).ok_or_else(|| {
        format!(
            "since \"{value}\" is not a time. Use an age such as 10m, 2h, or 1d, \
             a Unix time, or an RFC 3339 time."
        )
    })
}

#[cfg(test)]
mod tests;
