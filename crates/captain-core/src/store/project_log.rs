use std::collections::{HashSet, VecDeque};

use crate::model::{EngineEvent, EventKind, LogLine};

/// How many entries a project log keeps before it drops the oldest.
pub const PROJECT_LOG_LEN: usize = 2000;

/// One row of a project log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectLogEntry {
    /// A line of one service.
    Line { service: String, line: LogLine },
    /// A service's container stopped on its own or was killed.
    Exit(ServiceExit),
}

/// The divider a project log shows when a container exits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceExit {
    pub service: String,
    pub exit_code: Option<i64>,
    /// True if an `oom` event came before the exit.
    pub out_of_memory: bool,
    /// Unix seconds.
    pub time: Option<i64>,
}

impl ServiceExit {
    /// For example `worker exited 137 (out of memory)`.
    pub fn label(&self) -> String {
        let code = self
            .exit_code
            .map(|code| format!(" {code}"))
            .unwrap_or_default();
        let oom = if self.out_of_memory {
            " (out of memory)"
        } else {
            ""
        };
        format!("{} exited{code}{oom}", self.service)
    }
}

impl ProjectLogEntry {
    fn time(&self) -> Option<i64> {
        match self {
            Self::Line { line, .. } => line.timestamp,
            Self::Exit(exit) => exit.time,
        }
    }
}

/// The lines of every service of a project in one list, ordered by time, with a
/// divider where a container exited. It outlives the containers, so a restart
/// keeps the old lines.
#[derive(Debug, Clone, Default)]
pub struct ProjectLog {
    entries: VecDeque<ProjectLogEntry>,
    /// Containers with an `oom` event since their last exit.
    out_of_memory: HashSet<String>,
}

impl ProjectLog {
    pub fn push_line(&mut self, service: &str, line: LogLine) {
        self.insert(ProjectLogEntry::Line {
            service: service.to_string(),
            line,
        });
    }

    /// Adds an exit divider for a `die` event of the container that runs `service`,
    /// and remembers an `oom` event for the next exit. Other events change nothing.
    pub fn record(&mut self, service: &str, event: &EngineEvent) {
        if event.kind != EventKind::Container {
            return;
        }
        match event.action.as_str() {
            "oom" => {
                self.out_of_memory.insert(event.id.clone());
            }
            "die" => {
                let exit = ServiceExit {
                    service: service.to_string(),
                    exit_code: event.exit_code,
                    out_of_memory: self.out_of_memory.remove(&event.id),
                    time: event.time,
                };
                self.insert(ProjectLogEntry::Exit(exit));
            }
            _ => {}
        }
    }

    pub fn entries(&self) -> impl Iterator<Item = &ProjectLogEntry> {
        self.entries.iter()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.out_of_memory.clear();
    }

    /// Puts `entry` after the last entry of the same second or earlier, so entries
    /// of one second keep the order they came in. An entry without a time goes last.
    fn insert(&mut self, entry: ProjectLogEntry) {
        let at = match entry.time() {
            Some(time) => self
                .entries
                .iter()
                .rposition(|old| old.time().is_none_or(|old| old <= time))
                .map_or(0, |ix| ix + 1),
            None => self.entries.len(),
        };
        self.entries.insert(at, entry);
        if self.entries.len() > PROJECT_LOG_LEN {
            self.entries.pop_front();
        }
    }
}

#[cfg(test)]
mod tests;
