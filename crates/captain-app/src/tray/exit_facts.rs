//! What `inspect` said about each crashing container: whether the kernel killed it
//! for memory, its memory limit, and its restart count. The problem line needs them
//! to tell an out-of-memory crash and to offer Raise Memory.

use std::collections::HashMap;
use std::time::Instant;

use captain_core::model::ContainerDetail;
use captain_core::problems::{ExitFacts, MemoryRaises};

#[derive(Default)]
pub struct ExitFactsCache {
    details: HashMap<String, ContainerDetail>,
    /// Each crashing container, and the crash it was last inspected for: `None` for
    /// a container that restarts without a crash event.
    seen: HashMap<String, Option<Instant>>,
    /// The limits the menu raised, with the Project page's rule: no second raise
    /// until a run with the new limit starts.
    raises: MemoryRaises,
}

impl ExitFactsCache {
    /// Takes the containers that crash now, with their last crash. Returns the ones
    /// to inspect: each new crash asks again, so a raised limit shows. Forgets the
    /// containers that stopped crashing.
    pub fn follow(&mut self, crashing: HashMap<String, Option<Instant>>) -> Vec<String> {
        self.details.retain(|id, _| crashing.contains_key(id));
        let fresh = crashing
            .iter()
            .filter(|(id, at)| self.seen.get(*id) != Some(at))
            .map(|(id, _)| id.clone())
            .collect();
        self.seen = crashing;
        fresh
    }

    /// Stores what `inspect` said. True if it changed what the menu knows.
    pub fn insert(&mut self, id: String, detail: ContainerDetail) -> bool {
        if !self.seen.contains_key(&id) {
            return false;
        }
        let facts = self.facts_of(&detail);
        let old = self.details.insert(id, detail);
        old.map(|old| self.facts_of(&old)) != Some(facts)
    }

    /// Notes a raise of `id`'s limit, so the menu does not offer it again for the
    /// same run.
    pub fn record_raise(&mut self, id: &str) {
        if let Some(detail) = self.details.get(id) {
            self.raises.record(detail);
        }
    }

    /// Forgets a raise that failed, so the menu offers it again.
    pub fn forget_raise(&mut self, id: &str) {
        self.raises.forget(id);
    }

    pub fn facts(&self) -> HashMap<String, ExitFacts> {
        self.details
            .iter()
            .map(|(id, detail)| (id.clone(), self.facts_of(detail)))
            .collect()
    }

    fn facts_of(&self, detail: &ContainerDetail) -> ExitFacts {
        ExitFacts {
            raised: self.raises.raised(detail),
            ..ExitFacts::of(detail)
        }
    }
}

#[cfg(test)]
mod tests;
