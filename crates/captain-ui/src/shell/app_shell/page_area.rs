//! The area right of the rail and the sidebar: the page, the details panel beside
//! it, and the terminal panel under both.

use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use super::AppShell;
use crate::shell::details_rail::{self, Details};
use crate::theme::Palette;
use crate::workspace::Page;

impl AppShell {
    /// The page with its details, and under it the terminal panel while it is open.
    pub(super) fn page_area(
        &self,
        page: Page,
        details: Details,
        (sidebar_hidden, terminal_open): (bool, bool),
        palette: &Palette,
    ) -> Div {
        div()
            .flex_1()
            .min_w_0()
            .h_full()
            .flex()
            .flex_col()
            // The traffic lights are wider than the rail, so a page beside the
            // rail alone keeps its title clear of them.
            .child(
                self.page(page, details, palette)
                    .min_h_0()
                    .when(sidebar_hidden, |page| page.pl(px(16.))),
            )
            .when(terminal_open, |area| area.child(self.terminal.clone()))
    }

    /// The page, and beside the Containers and Project pages the details panel, or
    /// the rail that shows it again when `details` says it is hidden.
    fn page(&self, page: Page, details: Details, palette: &Palette) -> Div {
        let main = div().flex_1().min_w_0().h_full();
        let row = div().flex_1().min_w_0().flex();
        let details = |row: Div| match &details {
            Details::None => row,
            Details::Shown => row.child(self.inspector.clone()),
            Details::Hidden(name) => {
                row.child(details_rail::render(&self.workspace, name, palette))
            }
        };
        match page {
            Page::Containers => details(row.child(main.child(self.containers.clone()))),
            Page::Project => details(row.child(main.child(self.project.clone()))),
            Page::Images => row.child(main.child(self.images.clone())),
            Page::Volumes => row.child(main.child(self.volumes.clone())),
            Page::Networks => row.child(main.child(self.networks.clone())),
            Page::Extensions => row.child(main.child(self.extensions.clone())),
            Page::Snapshots => row.child(main.child(self.snapshots.clone())),
            Page::Storage => row.child(main.child(self.storage.clone())),
            Page::PortForwarding => row.child(main.child(self.forwarding.clone())),
            Page::Diagnostics => row.child(main.child(self.diagnostics.clone())),
            Page::Settings => row.child(main.child(self.settings.clone())),
        }
    }
}
