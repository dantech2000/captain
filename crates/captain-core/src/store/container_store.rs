use std::cmp::Ordering;

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
