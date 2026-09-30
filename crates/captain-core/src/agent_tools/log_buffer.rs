//! What `logs` keeps while it reads a container: only the lines that pass the
//! filters, each cut to a bounded length, and only the newest that could reach an
//! answer. A container that writes huge or many lines cannot fill the server's
//! memory.

use std::collections::VecDeque;

use super::log_query::{LogQuery, MAX_BYTES, MAX_LINE_CHARS, MAX_LINES};
use crate::model::LogLine;

/// The characters of a line kept while reading: twice the cut, so masking still
/// sees a whole secret that starts before the cut.
const KEPT_CHARS: usize = 2 * MAX_LINE_CHARS;
/// The bytes of text kept per container: more than one answer holds.
pub const KEPT_BYTES: usize = 2 * MAX_BYTES;

/// The newest matching lines of one container, and how many matched in all.
#[derive(Debug, Default)]
pub struct LogBuffer {
    lines: VecDeque<LogLine>,
    bytes: usize,
    matched: usize,
}

impl LogBuffer {
    /// Keeps `line` if `query` keeps it, and drops the oldest lines past the
    /// limits.
    pub fn push(&mut self, query: &LogQuery, mut line: LogLine) {
        if !query.keeps(&line) {
            return;
        }
        self.matched += 1;
        if let Some((end, _)) = line.text.char_indices().nth(KEPT_CHARS) {
            line.text.truncate(end);
        }
        self.bytes += line.text.len();
        self.lines.push_back(line);
        while self.lines.len() > MAX_LINES || self.bytes > KEPT_BYTES {
            let Some(old) = self.lines.pop_front() else {
                break;
            };
            self.bytes -= old.text.len();
        }
    }

    /// The lines that passed the filters, kept or not.
    pub fn matched(&self) -> usize {
        self.matched
    }

    /// The kept lines, oldest first.
    pub fn into_lines(self) -> Vec<LogLine> {
        self.lines.into()
    }
}

#[cfg(test)]
mod tests;
