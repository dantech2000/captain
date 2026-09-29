use gpui_kit::component::Icon;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use crate::theme::Palette;
use crate::workspace::{Page, Workspace};

/// The Settings entry above the engine card.
pub fn render(
    handle: &Entity<Workspace>,
    workspace: &Workspace,
    palette: &Palette,
) -> impl IntoElement {
    let selected = workspace.page() == Page::Settings;
    let handle = handle.clone();
    let hover = palette.nav_selected;
    div()
        .id("settings-link")
        .h(px(32.))
        .mb(px(8.))
        .px(px(10.))
        .flex()
        .items_center()
        .gap(px(10.))
        .rounded(px(8.))
        .cursor_pointer()
        .when(selected, |this| {
            this.bg(palette.nav_selected)
                .font_weight(FontWeight::SEMIBOLD)
        })
        .when(!selected, |this| this.hover(move |style| style.bg(hover)))
        .on_click(move |_, _, cx| {
            handle.update(cx, |workspace, cx| workspace.set_page(Page::Settings, cx));
        })
        .child(
            Icon::new(Page::Settings.icon())
                .size(px(16.))
                .text_color(if selected {
                    palette.accent
                } else {
                    palette.text2
                }),
        )
        .child(Page::Settings.label())
}
