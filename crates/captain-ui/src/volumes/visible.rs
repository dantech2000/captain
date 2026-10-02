//! The volumes the list shows. An extension's backend volumes hide with its
//! containers. See feature 0025.

use std::collections::HashSet;

use captain_core::model::Volume;
use captain_core::store::{ResourceGroup, UsageFilter, visible_groups};
use gpui_kit::*;

use super::VolumesView;

impl VolumesView {
    /// The groups the list shows, in order. A Shift-click selects in this order, so
    /// it cannot reach a hidden volume.
    pub(super) fn visible_groups(&self) -> Vec<ResourceGroup<Volume>> {
        visible_groups(self.store.groups(self.filter), self.show_extensions)
    }

    /// Follows `show_extension_containers`. Volumes that the list hides again leave
    /// the selection, so a bulk delete cannot reach them.
    pub(super) fn apply_settings(&mut self, cx: &mut Context<Self>) {
        let show = crate::settings::current(cx).show_extension_containers;
        if show == self.show_extensions {
            return;
        }
        self.show_extensions = show;
        let shown: HashSet<String> = visible_groups(self.store.groups(UsageFilter::All), show)
            .into_iter()
            .flat_map(|group| group.items.into_iter().map(|v| v.name))
            .collect();
        self.checked.retain(|name| shown.contains(name));
        if self
            .selected
            .as_ref()
            .is_some_and(|name| !shown.contains(name))
        {
            self.selected = None;
            self.load_users(cx);
        }
        cx.notify();
    }
}
