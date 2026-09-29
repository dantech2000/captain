use std::collections::VecDeque;

/// Keeps this many lines; a first start prints thousands.
const CAPACITY: usize = 200;

/// The latest progress lines of a start, oldest first.
#[derive(Debug, Clone, Default)]
pub struct ProgressLog {
    lines: VecDeque<String>,
}

impl ProgressLog {
    pub fn push(&mut self, line: String) {
        if self.lines.len() == CAPACITY {
            self.lines.pop_front();
        }
        self.lines.push_back(line);
    }

    pub fn clear(&mut self) {
        self.lines.clear();
    }

    /// The last `count` lines, oldest first.
    pub fn last(&self, count: usize) -> impl Iterator<Item = &str> {
        let skip = self.lines.len().saturating_sub(count);
        self.lines.iter().skip(skip).map(String::as_str)
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
}

#[cfg(test)]
mod tests;
