use captain_core::model::Image;
use gpui_kit::assets::IconName;
use gpui_kit::*;

use crate::images::{ImagesState, push_dialog, run_dialog, scan_dialog, tag_dialog};
use crate::theme::Palette;
use crate::widgets::action_button;

/// Run, Tag, and Push, then Scan, Remove, and Copy ID, as two rows of equal buttons.
/// Run waits for the details, because the dialog is prefilled from them. Push needs a
/// tag. Remove is off for an image in use.
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
        let generation = state.generation();
        action_button(
            "Remove",
            IconName::Trash,
            palette.red,
            !image.in_use() && !state.is_removing(&image.id),
            palette,
            move |_, cx| handle.update(cx, |state, cx| state.remove(id.clone(), generation, cx)),
        )
    };
    let tag = {
        let handle = handle.clone();
        let id = image.id.clone();
        let current = image.repo_tags.first().cloned().unwrap_or_default();
        action_button(
            "Tag",
            IconName::Tag,
            palette.accent,
            true,
            palette,
            move |window, cx| {
                tag_dialog::open(handle.clone(), id.clone(), current.clone(), window, cx);
            },
        )
    };
    let push = {
        let handle = handle.clone();
        let tags = image.repo_tags.clone();
        action_button(
            "Push",
            IconName::Upload,
            palette.accent,
            !tags.is_empty() && !state.is_pushing(),
            palette,
            move |window, cx| push_dialog::open(handle.clone(), tags.clone(), window, cx),
        )
    };
    let scan = {
        let engine = state.engine.clone();
        let reference = image
            .repo_tags
            .first()
            .cloned()
            .unwrap_or_else(|| image.id.clone());
        action_button(
            "Scan",
            IconName::ShieldCheck,
            palette.accent,
            engine.is_some(),
            palette,
            move |window, cx| {
                if let Some(engine) = engine.clone() {
                    scan_dialog::open(engine, reference.clone(), window, cx);
                }
            },
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
        .flex_col()
        .gap(px(8.))
        .child(div().flex().gap(px(8.)).child(run).child(tag).child(push))
        .child(
            div()
                .flex()
                .gap(px(8.))
                .child(scan)
                .child(remove)
                .child(copy),
        )
}
