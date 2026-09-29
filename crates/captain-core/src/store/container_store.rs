use std::cmp::Ordering;

use super::{ContainerFilter, ContainerGroup};
use crate::model::Container;

/// The container list that views render. It keeps containers in display order:
/// active containers first, then by name.
#[derive(Debug, Clone, Default)]
pub struct ContainerStore {
    containers: Vec<Container>,
}

impl ContainerStore {
    /// Replaces the whole list with a fresh one from the engine.
    pub fn replace(&mut self, mut containers: Vec<Container>) {
        containers.sort_by(display_order);
        self.containers = containers;
    }

    pub fn containers(&self) -> &[Container] {
        &self.containers
    }

    pub fn get(&self, index: usize) -> Option<&Container> {
        self.containers.get(index)
    }

    pub fn len(&self) -> usize {
        self.containers.len()
    }

    pub fn is_empty(&self) -> bool {
        self.containers.is_empty()
    }

    pub fn find(&self, id: &str) -> Option<&Container> {
        self.containers.iter().find(|c| c.id == id)
    }

    /// Containers that pass `filter`, grouped by Compose project. Projects come first,
    /// sorted by name. Standalone containers come last.
    pub fn groups(&self, filter: ContainerFilter) -> Vec<ContainerGroup> {
        let mut groups: Vec<ContainerGroup> = Vec::new();
        for container in self.containers.iter().filter(|c| filter.matches(c)) {
            let project = container.compose_project.clone();
            match groups.iter_mut().find(|g| g.project == project) {
                Some(group) => group.containers.push(container.clone()),
                None => groups.push(ContainerGroup {
                    project,
                    containers: vec![container.clone()],
                }),
            }
        }
        groups.sort_by(|a, b| match (&a.project, &b.project) {
            (Some(a), Some(b)) => a.cmp(b),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => Ordering::Equal,
        });
        groups
    }

    /// Every Compose project with its running and total container counts.
    pub fn projects(&self) -> Vec<ContainerGroup> {
        self.groups(ContainerFilter::All)
            .into_iter()
            .filter(|g| g.project.is_some())
            .collect()
    }

    /// The number of containers that are running, paused, or restarting.
    pub fn active_count(&self) -> usize {
        self.containers
            .iter()
            .filter(|c| c.state.is_active())
            .count()
    }
}

fn display_order(a: &Container, b: &Container) -> Ordering {
    b.state
        .is_active()
        .cmp(&a.state.is_active())
        .then_with(|| a.name.cmp(&b.name))
}

#[cfg(test)]
mod tests;
