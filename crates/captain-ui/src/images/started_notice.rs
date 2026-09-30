use gpui_kit::assets::IconName;
use gpui_kit::component::Icon;
use gpui_kit::*;

use super::{ImagesState, Started};
use crate::theme::Palette;
use crate::widgets::icon_button;
use crate::workspace::{Page, Workspace};

/// "Started web" in green, with a Show link that selects the container on the
/// Containers page, and a button that hides the notice.
pub fn render(
    started: &Started,
    handle: &Entity<ImagesState>,
    workspace: &Entity<Workspace>,
    palette: &Palette,
) -> Div {
    let show = {
        let handle = handle.clone();
        let workspace = workspace.clone();
        let id = started.id.clone();
        div()
            .id("show-started")
            .cursor_pointer()
            .text_color(palette.link)
            .font_weight(FontWeight::MEDIUM)
            .hover(|style| style.underline())
            .on_click(move |_, _, cx| {
                workspace.update(cx, |workspace, cx| {
                    workspace.select(id.clone(), cx);
                    workspace.set_page(Page::Containers, cx);
                });
                handle.update(cx, |state, cx| state.dismiss_started(cx));
            })
            .child("Show")
    };
    let dismiss = {
        let handle = handle.clone();
        icon_button(
            "dismiss-started",
            IconName::Close,
            "Hide this notice.",
            palette,
            move |_, _, cx| {
                handle.update(cx, |state, cx| state.dismiss_started(cx));
            },
        )
    };

    div()
        .flex()
        .items_center()
        .gap(px(8.))
        .text_size(px(12.))
        .child(
            Icon::new(IconName::CircleCheck)
                .size(px(14.))
                .text_color(palette.green),
        )
        .child(
            div()
                .text_color(palette.green)
                .child(format!("Started {}", started.name)),
        )
        .child(show)
        .child(dismiss)
}
