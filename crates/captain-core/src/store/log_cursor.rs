use std::collections::HashSet;

use crate::model::LogLine;

/// Where a container's log stream stopped, with full time precision, so a new
/// stream can resume there without lost or doubled lines.
///
/// Docker's `since` keeps lines at or after the time it names
/// (<https://github.com/moby/moby/blob/v28.5.0/daemon/logger/loggerutils/logfile.go#L831-L839>).
/// The daemon reads `since` as `seconds.nanoseconds`
/// (<https://github.com/moby/moby/blob/v28.5.0/daemon/logs.go>), but bollard 0.21
/// sends whole seconds only. So a resumed stream starts at the cursor's second, and
/// the cursor drops the lines it already saw.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LogCursor {
    /// The time of the newest line, as Unix seconds and nanoseconds.
    time: Option<(i64, u32)>,
    /// The texts of the lines seen at exactly `time`.
    seen: HashSet<String>,
}

impl LogCursor {
    /// The `since` to resume from: the second of the newest line.
    pub fn since(&self) -> Option<i64> {
        self.time.map(|(seconds, _)| seconds)
    }

    /// Moves the cursor to `line`. Returns false if the line is a replay: older than
    /// the newest line, or the same time and text as a line already seen. A line
    /// with no time is always new.
    pub fn advance(&mut self, line: &LogLine) -> bool {
        let Some(time) = line.precise_time() else {
            return true;
        };
        match self.time {
            Some(newest) if time < newest => false,
            Some(newest) if time == newest => self.seen.insert(line.text.clone()),
            _ => {
                self.time = Some(time);
                self.seen.clear();
                self.seen.insert(line.text.clone());
                true
            }
        }
    }
}

#[cfg(test)]
mod tests;
