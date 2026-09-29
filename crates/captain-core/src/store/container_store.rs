use std::cmp::Ordering;

use super::{ContainerFilter, ContainerGroup, GroupKey};
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

    /// Containers that pass `filter`, in cards sorted by [`GroupKey`]. Kubernetes pod
    /// containers are left out unless `kubernetes` is true; then each namespace gets
    /// a card.
    pub fn groups(&self, filter: ContainerFilter, kubernetes: bool) -> Vec<ContainerGroup> {
        let mut groups: Vec<ContainerGroup> = Vec::new();
        let shown = self.shown(kubernetes).filter(|c| filter.matches(c));
        for container in shown {
            let key = GroupKey::of(container);
            match groups.iter_mut().find(|g| g.key == key) {
                Some(group) => group.containers.push(container.clone()),
                None => groups.push(ContainerGroup {
                    key,
                    containers: vec![container.clone()],
                }),
            }
        }
        groups.sort_by(|a, b| a.key.cmp(&b.key));
        groups
    }

    /// Every Compose project with its running and total container counts.
    pub fn projects(&self) -> Vec<ContainerGroup> {
        self.groups(ContainerFilter::All, false)
            .into_iter()
            .filter(|g| g.project().is_some())
            .collect()
    }

    /// The containers the list shows: all of them, or all but the Kubernetes pod
    /// containers.
    pub fn shown(&self, kubernetes: bool) -> impl Iterator<Item = &Container> {
        self.containers
            .iter()
            .filter(move |c| kubernetes || !c.is_kubernetes())
    }

    /// The number of Kubernetes pod containers.
    pub fn kubernetes_count(&self) -> usize {
        self.containers.iter().filter(|c| c.is_kubernetes()).count()
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
