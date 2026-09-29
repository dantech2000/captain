use std::collections::VecDeque;

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
    pub fn push(&mut self, line: LogLine) {
        if self.lines.len() == LOG_BUFFER_LEN {
            self.lines.pop_front();
        }
        self.lines.push_back(line);
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
}

#[cfg(test)]
mod tests;
