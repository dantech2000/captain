use gpui_kit::component::Icon;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use crate::theme::Palette;
use crate::workspace::{Page, Workspace};

/// A page entry above the engine card: Diagnostics or Settings. `badge` is a count
/// of problems, shown in red.
pub fn render(
    handle: &Entity<Workspace>,
    workspace: &Workspace,
    page: Page,
    badge: Option<usize>,
    palette: &Palette,
) -> Stateful<Div> {
    let selected = workspace.page() == page;
    let handle = handle.clone();
    let hover = palette.nav_selected;
    div()
        .id(SharedString::from(format!("page-link-{}", page.label())))
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
        .on_click(move |_, _, cx| {
            handle.update(cx, |workspace, cx| workspace.set_page(page, cx));
        })
        .child(
            Icon::new(page.icon())
                .size(px(16.))
                .text_color(if selected {
                    palette.accent
                } else {
                    palette.text2
                }),
        )
        .child(div().flex_1().child(page.label()))
        .children(badge.map(|count| {
            div()
                .min_w(px(18.))
                .h(px(18.))
                .px(px(5.))
                .flex()
                .items_center()
                .justify_center()
                .rounded_full()
                .bg(palette.red)
                .text_color(white())
                .text_size(px(11.))
                .font_weight(FontWeight::SEMIBOLD)
                .child(count.to_string())
        }))
}
