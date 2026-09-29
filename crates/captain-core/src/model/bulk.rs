use crate::EngineError;

/// What one action on several containers or volumes did.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BulkOutcome {
    /// The names the action finished on.
    pub done: Vec<String>,
    /// The names it failed on, with the engine's error.
    pub failed: Vec<(String, EngineError)>,
}

impl BulkOutcome {
    pub fn total(&self) -> usize {
        self.done.len() + self.failed.len()
    }

    /// For example `Deleted 3 containers.`
    pub fn done_message(&self, verb_past: &str, noun: &str) -> String {
        format!("{verb_past} {}.", count_label(self.done.len(), noun))
    }

    /// For example `Could not stop 2 of 5 containers`.
    pub fn failed_title(&self, verb: &str, noun: &str) -> String {
        format!(
            "Could not {} {} of {}",
            verb.to_lowercase(),
            self.failed.len(),
            count_label(self.total(), noun)
        )
    }

    /// One line per failure: the name and the engine's message.
    pub fn failed_lines(&self) -> String {
        self.failed
            .iter()
            .map(|(name, error)| format!("{name}: {error}"))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// `1 container`, `2 containers`.
pub fn count_label(count: usize, noun: &str) -> String {
    if count == 1 {
        format!("1 {noun}")
    } else {
        format!("{count} {noun}s")
    }
}

#[cfg(test)]
mod tests;
