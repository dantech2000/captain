use captain_core::model::Image;
use gpui_kit::assets::IconName;
use gpui_kit::*;

use crate::images::{ImagesState, run_dialog};
use crate::theme::Palette;
use crate::widgets::action_button;

/// Run, Remove, and Copy ID, as three equal buttons. Run waits for the details,
/// because the dialog is prefilled from them. Remove is off for an image in use.
pub fn render(
    image: &Image,
    handle: &Entity<ImagesState>,
    state: &ImagesState,
    palette: &Palette,
) -> impl IntoElement {
    let run = {
        let handle = handle.clone();
        let reference = image
            .repo_tags
            .first()
            .cloned()
            .unwrap_or_else(|| image.id.clone());
        let detail = state.detail().cloned();
        action_button(
            "Run",
            IconName::Play,
            palette.accent,
            detail.is_some(),
            palette,
            move |window, cx| {
                if let Some(detail) = &detail {
                    run_dialog::open(handle.clone(), reference.clone(), detail, window, cx);
                }
            },
        )
    };
    let remove = {
        let handle = handle.clone();
        let id = image.id.clone();
        action_button(
            "Remove",
            IconName::Trash,
            palette.red,
            !image.in_use() && !state.is_removing(&image.id),
            palette,
            move |_, cx| handle.update(cx, |state, cx| state.remove(id.clone(), cx)),
        )
    };
    let copy = {
        let id = image.id.clone();
        action_button(
            "Copy ID",
            IconName::Copy,
            palette.text,
            true,
            palette,
            move |_, cx| cx.write_to_clipboard(ClipboardItem::new_string(id.clone())),
        )
    };

    div()
        .flex()
        .gap(px(8.))
        .child(run)
        .child(remove)
        .child(copy)
}
