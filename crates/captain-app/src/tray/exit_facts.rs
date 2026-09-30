//! What `inspect` said about each crashing container: whether the kernel killed it
//! for memory, its memory limit, and its restart count. The problem line needs them
//! to tell an out-of-memory crash and to offer Raise Memory.

use std::collections::HashMap;
use std::time::Instant;

use captain_core::problems::ExitFacts;

#[derive(Default)]
pub struct ExitFactsCache {
    facts: HashMap<String, ExitFacts>,
    /// Each crashing container, and the crash it was last inspected for: `None` for
    /// a container that restarts without a crash event.
    seen: HashMap<String, Option<Instant>>,
}

impl ExitFactsCache {
    /// Takes the containers that crash now, with their last crash. Returns the ones
    /// to inspect: each new crash asks again, so a raised limit shows. Forgets the
    /// containers that stopped crashing.
    pub fn follow(&mut self, crashing: HashMap<String, Option<Instant>>) -> Vec<String> {
        self.facts.retain(|id, _| crashing.contains_key(id));
        let fresh = crashing
            .iter()
            .filter(|(id, at)| self.seen.get(*id) != Some(at))
            .map(|(id, _)| id.clone())
            .collect();
        self.seen = crashing;
        fresh
    }

    /// Stores what `inspect` said. True if it changed what the menu knows.
    pub fn insert(&mut self, id: String, facts: ExitFacts) -> bool {
        if !self.seen.contains_key(&id) {
            return false;
        }
        self.facts.insert(id, facts) != Some(facts)
    }

    pub fn facts(&self) -> &HashMap<String, ExitFacts> {
        &self.facts
    }
}

#[cfg(test)]
mod tests;
