//! The networks the list shows. An extension's backend networks hide with its
//! containers. See feature 0025.

use captain_core::model::Network;
use captain_core::store::{ResourceGroup, UsageFilter, visible_groups};
use gpui_kit::*;

use super::NetworksView;

impl NetworksView {
    /// The groups the list shows, in order.
    pub(super) fn visible_groups(&self) -> Vec<ResourceGroup<Network>> {
        visible_groups(self.store.groups(self.filter), self.show_extensions)
    }

    /// Follows `show_extension_containers`. A network that the list hides again
    /// leaves the selection.
    pub(super) fn apply_settings(&mut self, cx: &mut Context<Self>) {
        let show = crate::settings::current(cx).show_extension_containers;
        if show == self.show_extensions {
            return;
        }
        self.show_extensions = show;
        let hidden = self.selected.as_deref().is_some_and(|id| {
            !visible_groups(self.store.groups(UsageFilter::All), show)
                .iter()
                .any(|group| group.items.iter().any(|n| n.id == id))
        });
        if hidden {
            self.selected = None;
            self.detail = None;
            self.detail_task = None;
        }
        cx.notify();
    }
}
