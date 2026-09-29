use super::{MigrationItem, Step};

/// Which images the plan copies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ImageChoice {
    #[default]
    All,
    /// Only images that a container uses. This keeps a migration short.
    InUse,
}

/// An item and the user's choices for it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanEntry {
    pub item: MigrationItem,
    pub selected: bool,
    /// Copy the container's own filesystem with `docker commit`. Opt-in, and only
    /// for containers that have changes. It writes a temporary image in the source.
    pub snapshot: bool,
}

/// What the source engine holds and what the user picked. Everything starts selected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationPlan {
    /// The source engine, for example `unix:///Users/me/.rd/docker.sock`.
    pub source: String,
    /// Entries in step order.
    pub entries: Vec<PlanEntry>,
    images: ImageChoice,
}

impl MigrationPlan {
    /// A plan with every item selected, sorted by step and then by label.
    pub fn new(source: impl Into<String>, mut items: Vec<MigrationItem>) -> Self {
        items.sort_by_key(|item| (item.step(), item.label()));
        let entries = items
            .into_iter()
            .map(|item| PlanEntry {
                item,
                selected: true,
                snapshot: false,
            })
            .collect();
        Self {
            source: source.into(),
            entries,
            images: ImageChoice::All,
        }
    }

    pub fn image_choice(&self) -> ImageChoice {
        self.images
    }

    /// Selects all images, or only the ones in use. It replaces image selections the
    /// user made by hand.
    pub fn set_image_choice(&mut self, choice: ImageChoice) {
        self.images = choice;
        for entry in &mut self.entries {
            if let MigrationItem::Image { in_use, .. } = entry.item {
                entry.selected = choice == ImageChoice::All || in_use;
            }
        }
    }

    /// Selects or clears the entry with `key`.
    pub fn toggle(&mut self, key: &str) {
        if let Some(entry) = self.find_mut(key) {
            entry.selected = !entry.selected;
        }
    }

    /// Turns the snapshot on or off. Only items that lose changes can have one.
    pub fn set_snapshot(&mut self, key: &str, snapshot: bool) {
        if let Some(entry) = self.find_mut(key) {
            entry.snapshot = snapshot && matches!(entry.item, MigrationItem::Container { .. });
        }
    }

    /// The entries of one step.
    pub fn step(&self, step: Step) -> impl Iterator<Item = &PlanEntry> {
        self.entries.iter().filter(move |e| e.item.step() == step)
    }

    /// The selected entries, in the order they run.
    pub fn selected(&self) -> impl Iterator<Item = &PlanEntry> {
        self.entries.iter().filter(|e| e.selected)
    }

    /// Bytes the selected entries copy, snapshots included.
    pub fn total_bytes(&self) -> u64 {
        self.selected()
            .map(|e| {
                e.item.size()
                    + if e.snapshot {
                        e.item.changed_bytes()
                    } else {
                        0
                    }
            })
            .sum()
    }

    /// Selected items that lose changes made inside a container and have no snapshot.
    pub fn lost_changes(&self) -> Vec<&MigrationItem> {
        self.selected()
            .filter(|e| e.item.loses_changes() && !e.snapshot)
            .map(|e| &e.item)
            .collect()
    }

    /// Selected volumes that running containers use, with those containers' names.
    pub fn live_volumes(&self) -> Vec<(&str, &[String])> {
        self.selected()
            .filter_map(|e| match &e.item {
                MigrationItem::Volume {
                    name,
                    used_by_running,
                    ..
                } if !used_by_running.is_empty() => {
                    Some((name.as_str(), used_by_running.as_slice()))
                }
                _ => None,
            })
            .collect()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn find_mut(&mut self, key: &str) -> Option<&mut PlanEntry> {
        self.entries.iter_mut().find(|e| e.item.key() == key)
    }
}

#[cfg(test)]
mod tests;
