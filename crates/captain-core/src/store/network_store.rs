use super::{ResourceGroup, UsageFilter, group_by_project};
use crate::model::Network;

/// The network list that views render: built-in networks first, then by name.
#[derive(Debug, Clone, Default)]
pub struct NetworkStore {
    networks: Vec<Network>,
}

impl NetworkStore {
    /// Replaces the whole list with a fresh one from the engine.
    pub fn replace(&mut self, mut networks: Vec<Network>) {
        networks.sort_by(|a, b| {
            b.is_built_in()
                .cmp(&a.is_built_in())
                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });
        self.networks = networks;
    }

    pub fn networks(&self) -> &[Network] {
        &self.networks
    }

    pub fn len(&self) -> usize {
        self.networks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.networks.is_empty()
    }

    pub fn find(&self, id: &str) -> Option<&Network> {
        self.networks.iter().find(|n| n.id == id)
    }

    /// The number of networks that pass `filter`.
    pub fn count(&self, filter: UsageFilter) -> usize {
        self.networks.iter().filter(|n| filter.matches(*n)).count()
    }

    /// Networks that pass `filter`, grouped by Compose project.
    pub fn groups(&self, filter: UsageFilter) -> Vec<ResourceGroup<Network>> {
        group_by_project(self.networks.iter().filter(|n| filter.matches(*n)), |n| {
            n.compose_project.as_deref()
        })
    }

    /// For example `4 networks · 2 in use`.
    pub fn summary(&self) -> String {
        let count = match self.networks.len() {
            1 => "1 network".to_string(),
            n => format!("{n} networks"),
        };
        match self.count(UsageFilter::InUse) {
            0 => count,
            n => format!("{count} · {n} in use"),
        }
    }

    /// The first network in display order that passes `filter`.
    pub fn first(&self, filter: UsageFilter) -> Option<&Network> {
        self.groups(filter)
            .into_iter()
            .next()
            .and_then(|g| g.items.into_iter().next())
            .and_then(|n| self.find(&n.id))
    }
}

#[cfg(test)]
mod tests;
