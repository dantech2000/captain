use super::{ResourceGroup, UsageFilter, group_by_project};
use crate::format::bytes_label;
use crate::model::Volume;

/// The volume list that views render, sorted by name with anonymous volumes last.
#[derive(Debug, Clone, Default)]
pub struct VolumeStore {
    volumes: Vec<Volume>,
}

impl VolumeStore {
    /// Replaces the whole list with a fresh one from the engine.
    pub fn replace(&mut self, mut volumes: Vec<Volume>) {
        volumes.sort_by(|a, b| {
            a.is_anonymous()
                .cmp(&b.is_anonymous())
                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });
        self.volumes = volumes;
    }

    pub fn volumes(&self) -> &[Volume] {
        &self.volumes
    }

    pub fn len(&self) -> usize {
        self.volumes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.volumes.is_empty()
    }

    pub fn find(&self, name: &str) -> Option<&Volume> {
        self.volumes.iter().find(|v| v.name == name)
    }

    /// The number of volumes that pass `filter`.
    pub fn count(&self, filter: UsageFilter) -> usize {
        self.volumes.iter().filter(|v| filter.matches(*v)).count()
    }

    /// Volumes that pass `filter`, grouped by Compose project.
    pub fn groups(&self, filter: UsageFilter) -> Vec<ResourceGroup<Volume>> {
        group_by_project(self.volumes.iter().filter(|v| filter.matches(*v)), |v| {
            v.compose_project.as_deref()
        })
    }

    /// The disk usage of all volumes. `None` if the engine reported no sizes.
    pub fn total_size(&self) -> Option<u64> {
        self.volumes
            .iter()
            .filter_map(|v| v.size_bytes)
            .fold(None, |total, size| Some(total.unwrap_or(0) + size))
    }

    /// For example `6 volumes · 1.4 GB`.
    pub fn summary(&self) -> String {
        let count = match self.volumes.len() {
            1 => "1 volume".to_string(),
            n => format!("{n} volumes"),
        };
        match self.total_size() {
            Some(size) => format!("{count} · {}", bytes_label(size)),
            None => count,
        }
    }

    /// The first volume in display order that passes `filter`.
    pub fn first(&self, filter: UsageFilter) -> Option<&Volume> {
        self.groups(filter)
            .into_iter()
            .next()
            .and_then(|g| g.items.into_iter().next())
            .and_then(|v| self.find(&v.name))
    }
}

#[cfg(test)]
mod tests;
