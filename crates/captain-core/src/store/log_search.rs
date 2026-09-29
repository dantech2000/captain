use std::ops::Range;

use crate::model::LogLine;

/// A log line that passed the Logs tab filters, with where the search text matched.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogMatch {
    pub line: LogLine,
    /// Byte ranges into `line.text`, left to right. Empty when there is no search text.
    pub ranges: Vec<Range<usize>>,
}

/// Finds each case-insensitive occurrence of `query` in `text`, left to right, as
/// byte ranges that do not overlap. An empty query matches with no ranges. Returns
/// `None` when a non-empty query does not occur.
pub fn find_matches(text: &str, query: &str) -> Option<Vec<Range<usize>>> {
    if query.is_empty() {
        return Some(Vec::new());
    }
    let mut ranges = Vec::new();
    let mut from = 0;
    for (start, _) in text.char_indices() {
        if start < from {
            continue;
        }
        if let Some(end) = match_at(text, start, query) {
            ranges.push(start..end);
            from = end;
        }
    }
    (!ranges.is_empty()).then_some(ranges)
}

/// The end of `query` when it matches `text` at byte `start`, ignoring case.
fn match_at(text: &str, start: usize, query: &str) -> Option<usize> {
    let mut chars = text[start..].char_indices();
    let mut end = start;
    for wanted in query.chars() {
        let (offset, c) = chars.next()?;
        if fold(c) != fold(wanted) {
            return None;
        }
        end = start + offset + c.len_utf8();
    }
    Some(end)
}

fn fold(c: char) -> char {
    c.to_lowercase().next().unwrap_or(c)
}

#[cfg(test)]
mod tests;
