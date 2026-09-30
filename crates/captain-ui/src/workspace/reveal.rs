//! Pages that an extension window asks the main window to show, for
//! `ddClient.desktopUI.navigate`. See docs/features/0025-extensions.md.

use captain_core::extension::{ContainerView, NavigateIntent};
use captain_core::store::GroupKey;
use gpui_kit::*;

use super::{InspectorTab, Page, Workspace};

/// An image or a volume that its page selects once it shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reveal {
    Image(String),
    Volume(String),
}

impl Workspace {
    /// Opens the page for an extension's navigate call. `found` is the full ID or
    /// name the engine answered for a call that names a container, image, or volume.
    pub fn navigate(&mut self, intent: &NavigateIntent, found: String, cx: &mut Context<Self>) {
        match intent {
            NavigateIntent::Containers => self.set_page(Page::Containers, cx),
            NavigateIntent::Container { view, .. } => {
                self.show_container(found, InspectorTab::from(*view), cx)
            }
            NavigateIntent::Images => self.set_page(Page::Images, cx),
            NavigateIntent::Image { .. } => self.reveal(Reveal::Image(found), cx),
            NavigateIntent::Volumes => self.set_page(Page::Volumes, cx),
            NavigateIntent::Volume(_) => self.reveal(Reveal::Volume(found), cx),
        }
    }

    /// Shows container `id` with the inspector at `tab`: on the Project page of its
    /// sidebar entry, or on the Containers page while the list does not have it yet.
    fn show_container(&mut self, id: String, tab: InspectorTab, cx: &mut Context<Self>) {
        match self.store.find(&id).map(GroupKey::of) {
            Some(key) => self.open_group(key, cx),
            None => self.page = Page::Containers,
        }
        self.open_card_tab(id, tab, cx);
    }

    /// Shows the Images or Volumes page, which then selects the item.
    fn reveal(&mut self, reveal: Reveal, cx: &mut Context<Self>) {
        self.page = match reveal {
            Reveal::Image(_) => Page::Images,
            Reveal::Volume(_) => Page::Volumes,
        };
        self.reveal = Some(reveal);
        cx.notify();
    }

    /// The image to select, once. It does not notify.
    pub fn take_revealed_image(&mut self) -> Option<String> {
        match self.reveal.take() {
            Some(Reveal::Image(id)) => Some(id),
            other => {
                self.reveal = other;
                None
            }
        }
    }

    /// The volume to select, once. It does not notify.
    pub fn take_revealed_volume(&mut self) -> Option<String> {
        match self.reveal.take() {
            Some(Reveal::Volume(name)) => Some(name),
            other => {
                self.reveal = other;
                None
            }
        }
    }
}

impl From<ContainerView> for InspectorTab {
    fn from(view: ContainerView) -> Self {
        match view {
            ContainerView::Details => Self::Overview,
            ContainerView::Logs => Self::Logs,
            ContainerView::Terminal => Self::Terminal,
            ContainerView::Stats => Self::Stats,
        }
    }
}
