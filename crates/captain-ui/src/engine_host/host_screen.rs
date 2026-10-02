//! What the Containers page shows instead of the list while Captain Engine is not
//! running: setup, progress, or a stopped state.

use captain_core::HostStatus;
use gpui_kit::component::scroll::{Scrollable, ScrollableElement};
use gpui_kit::*;

use super::{HostModel, setup_screen, starting_screen, stopped_screen};
use crate::theme::Palette;
use crate::widgets::brand_mark;

/// The screen for the current host status, or `None` when the engine runs or the
/// settings choose another engine.
pub fn render(model: &Entity<HostModel>, palette: &Palette, cx: &App) -> Option<AnyElement> {
    let host = model.read(cx);
    if !host.uses_captain(cx) {
        return None;
    }
    if host.is_checking() {
        return Some(checking(palette).into_any_element());
    }
    let screen = match host.status() {
        HostStatus::Running => return None,
        HostStatus::NotCreated => setup_screen::render(model, host, palette).into_any_element(),
        HostStatus::Starting | HostStatus::Stopping => {
            starting_screen::render(host, palette).into_any_element()
        }
        HostStatus::Stopped | HostStatus::Failed(_) | HostStatus::NotInstalled(_) => {
            stopped_screen::render(model, host, palette).into_any_element()
        }
    };
    Some(screen)
}

/// A centered column with the Captain mark on top, shared by the host screens.
pub fn frame(palette: &Palette) -> Div {
    div()
        .w(px(540.))
        .flex()
        .flex_col()
        .items_center()
        .gap(px(14.))
        .text_color(palette.text)
        .child(brand_mark(px(88.)))
}

/// The page around a [`frame`] column.
pub fn page(column: Div) -> Scrollable<Stateful<Div>> {
    div()
        .id("host-screen")
        .size_full()
        .overflow_y_scrollbar()
        .flex()
        .justify_center()
        .items_center()
        .p(px(40.))
        .child(column)
}

pub fn title(text: impl Into<SharedString>) -> Div {
    div()
        .text_size(px(22.))
        .font_weight(FontWeight::BOLD)
        .child(text.into())
}

pub fn note(text: impl Into<SharedString>, palette: &Palette) -> Div {
    div()
        .text_color(palette.text2)
        .text_center()
        .child(text.into())
}

fn checking(palette: &Palette) -> impl IntoElement {
    page(frame(palette).child(note("Checking Captain Engine...", palette)))
}
