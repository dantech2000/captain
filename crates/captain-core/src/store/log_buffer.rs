use std::collections::VecDeque;

use super::log_search::{LogMatch, find_matches};
use crate::model::{LogLevel, LogLine};

/// How many lines a buffer keeps before it drops the oldest.
pub const LOG_BUFFER_LEN: usize = 2000;

/// Which log lines the Logs tab shows.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LevelFilter {
    #[default]
    All,
    Only(LogLevel),
}

impl LevelFilter {
    pub const ALL: [LevelFilter; 4] = [
        Self::All,
        Self::Only(LogLevel::Info),
        Self::Only(LogLevel::Warn),
        Self::Only(LogLevel::Error),
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Only(LogLevel::Info) => "Info",
            Self::Only(LogLevel::Warn) => "Warn",
            Self::Only(LogLevel::Error) => "Error",
        }
    }

    pub fn matches(self, line: &LogLine) -> bool {
        match self {
            Self::All => true,
            Self::Only(level) => line.level == level,
        }
    }
}

/// The recent output of one container, oldest first.
#[derive(Debug, Clone, Default)]
pub struct LogBuffer {
    lines: VecDeque<LogLine>,
}

impl LogBuffer {
    /// Adds `line`, and returns the oldest line when the buffer was full and
    /// dropped it.
    pub fn push(&mut self, line: LogLine) -> Option<LogLine> {
        let evicted = if self.lines.len() == LOG_BUFFER_LEN {
            self.lines.pop_front()
        } else {
            None
        };
        self.lines.push_back(line);
        evicted
    }

    pub fn clear(&mut self) {
        self.lines.clear();
    }

    pub fn len(&self) -> usize {
        self.lines.len()
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// The lines that pass `filter`, oldest first.
    pub fn filtered(&self, filter: LevelFilter) -> Vec<LogLine> {
        self.lines
            .iter()
            .filter(|l| filter.matches(l))
            .cloned()
            .collect()
    }

    /// The lines that pass `filter` and contain `query`, oldest first, with where
    /// `query` matched. See [`find_matches`].
    pub fn search(&self, filter: LevelFilter, query: &str) -> Vec<LogMatch> {
        self.lines
            .iter()
            .filter(|line| filter.matches(line))
            .filter_map(|line| {
                find_matches(&line.text, query).map(|ranges| LogMatch {
                    line: line.clone(),
                    ranges,
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests;
