use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use super::page_help::page_help;
use crate::help::HelpExt;
use crate::icons::glyph;
use crate::theme::Palette;
use crate::workspace::{Page, Workspace};

/// The pages. The current one is highlighted; a click switches pages.
pub fn render(
    handle: &Entity<Workspace>,
    workspace: &Workspace,
    palette: &Palette,
) -> impl IntoElement {
    let current = workspace.page();
    div()
        .flex()
        .flex_col()
        .gap(px(2.))
        .children(Page::ALL.into_iter().map(|page| {
            let selected = page == current;
            let count = workspace.page_count(page);
            let handle = handle.clone();
            let hover = palette.nav_selected;
            div()
                .id(page.label())
                .h(px(32.))
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
                .text_color(palette.text)
                .on_click(move |_, _, cx| {
                    handle.update(cx, |workspace, cx| workspace.set_page(page, cx))
                })
                .child(glyph(
                    page.icon(),
                    px(16.),
                    if selected {
                        palette.accent
                    } else {
                        palette.text2
                    },
                ))
                .child(div().flex_1().child(page.label()))
                .help(page_help(page, count))
                .children(count.map(|count| {
                    div()
                        .text_size(px(11.))
                        .font_weight(FontWeight::NORMAL)
                        .text_color(palette.text3)
                        .child(count.to_string())
                }))
        }))
}
