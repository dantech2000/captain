//! The Push dialog: the image's tags, each with a Push button.

use gpui_kit::component::WindowExt;
use gpui_kit::*;

use super::ImagesState;
use crate::theme::Palette;
use crate::widgets::{ButtonTone, section_note, text_button};

/// Opens the Push dialog for `tags`. A click on Push starts the push on the page and
/// closes the dialog.
pub fn open(state: Entity<ImagesState>, tags: Vec<String>, window: &mut Window, cx: &mut App) {
    window.open_dialog(cx, move |dialog, _, cx| {
        let palette = Palette::of(cx);
        let rows = tags.iter().enumerate().map(|(ix, tag)| {
            let state = state.clone();
            let reference = tag.clone();
            div()
                .flex()
                .items_center()
                .gap(px(8.))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .truncate()
                        .font_family(palette.mono())
                        .text_size(px(12.))
                        .child(tag.clone()),
                )
                .child(text_button(
                    ("push-tag", ix),
                    "Push",
                    ButtonTone::Accent,
                    true,
                    &palette,
                    move |_, window, cx| {
                        let reference = reference.clone();
                        state.update(cx, |state, cx| state.push_image(reference, cx));
                        window.close_dialog(cx);
                    },
                ))
        });
        dialog.title("Push image").w(px(460.)).child(
            div()
                .flex()
                .flex_col()
                .gap(px(10.))
                .pb(px(4.))
                .child(section_note(
                    "Captain uses your docker login. Add a tag with the registry first, \
                     for example ghcr.io/team/app:1.0.",
                    &palette,
                ))
                .children(rows),
        )
    });
}
