use std::sync::Arc;

use captain_core::ProjectRunner;
use captain_core::store::{ContainerFilter, ContainerGroup, GroupKey, StatsBoard};
use gpui_kit::*;

use super::{Page, Workspace};

/// An inspector tab that a button outside the inspector can open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InspectorTab {
    Logs,
    Terminal,
}

impl Workspace {
    /// The sidebar entry the Project page shows.
    pub fn focus(&self) -> Option<&GroupKey> {
        self.focus.as_ref()
    }

    /// Shows the Project page for `key`, with the inspector closed.
    pub fn open_group(&mut self, key: GroupKey, cx: &mut Context<Self>) {
        self.focus = Some(key);
        self.card_open = false;
        self.page = Page::Project;
        cx.notify();
    }

    /// The containers of the focused entry, Kubernetes ones included, in list order.
    pub fn focused_group(&self) -> Option<ContainerGroup> {
        let key = self.focus.as_ref()?;
        self.store
            .groups(ContainerFilter::All, true)
            .into_iter()
            .find(|group| &group.key == key)
    }

    /// The sidebar entries: Compose projects, then Kubernetes namespaces while their
    /// containers show, then the loose containers.
    pub fn sidebar_groups(&self) -> Vec<ContainerGroup> {
        self.store
            .groups(ContainerFilter::All, self.show_kubernetes)
    }

    /// True while the Project page shows the inspector.
    pub fn card_open(&self) -> bool {
        self.card_open
    }

    /// Opens the inspector on container `id`, or closes it if it shows `id` already.
    pub fn toggle_card(&mut self, id: String, cx: &mut Context<Self>) {
        let showing = self.card_open && self.selected.as_deref() == Some(id.as_str());
        if showing {
            self.card_open = false;
            cx.notify();
        } else {
            self.card_open = true;
            self.select(id, cx);
        }
    }

    /// Opens the inspector on container `id` at `tab`.
    pub fn open_card_tab(&mut self, id: String, tab: InspectorTab, cx: &mut Context<Self>) {
        self.card_open = true;
        self.inspector_tab = Some(tab);
        self.select(id, cx);
    }

    /// The tab a card button asked for, once. It does not notify.
    pub fn take_inspector_tab(&mut self) -> Option<InspectorTab> {
        self.inspector_tab.take()
    }

    /// The Compose runner, if `docker compose` is installed.
    pub fn project_runner(&self) -> Option<Arc<dyn ProjectRunner>> {
        self.projects.clone()
    }

    /// The stats of the containers the lists show, so hidden Kubernetes containers
    /// do not count in the totals.
    pub fn shown_stats(&self) -> StatsBoard {
        if self.show_kubernetes {
            return self.stats.clone();
        }
        let store = &self.store;
        self.stats
            .only(|id| store.find(id).is_some_and(|c| !c.is_kubernetes()))
    }
}
