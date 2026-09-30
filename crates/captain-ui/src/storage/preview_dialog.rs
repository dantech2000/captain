//! Lists every item the cleanup removes, by group, and runs it on confirm.

use captain_core::format::bytes_label;
use captain_core::storage::{ReclaimGroup, ReclaimItem};
use gpui_kit::component::WindowExt;
use gpui_kit::*;

use super::StorageModel;
use crate::theme::Palette;
use crate::widgets::{ButtonTone, text_button};

struct PreviewDialog {
    model: Entity<StorageModel>,
    items: Vec<ReclaimItem>,
    snapshot_first: bool,
}

impl Render for PreviewDialog {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = Palette::of(cx);
        let total: u64 = self.items.iter().map(|item| item.size).sum();
        let this = cx.entity().downgrade();
        let remove = text_button(
            "storage-preview-ok",
            format!(
                "Remove {} and free {}",
                count(self.items.len()),
                bytes_label(total)
            ),
            ButtonTone::Danger,
            true,
            &palette,
            move |_, window, cx| {
                this.update(cx, |dialog, cx| {
                    let items = std::mem::take(&mut dialog.items);
                    dialog
                        .model
                        .update(cx, |model, cx| model.clean_up(items, cx));
                })
                .ok();
                window.close_dialog(cx);
            },
        );
        let cancel = text_button(
            "storage-preview-cancel",
            "Cancel",
            ButtonTone::Accent,
            true,
            &palette,
            |_, window, cx| window.close_dialog(cx),
        );
        let note = if self.snapshot_first {
            "Captain Engine saves a snapshot first and restarts. Then these items are removed."
        } else {
            "These items are removed. Volumes cannot come back without a snapshot."
        };
        let groups = ReclaimGroup::ALL.into_iter().filter_map(|group| {
            let items: Vec<&ReclaimItem> = self
                .items
                .iter()
                .filter(|item| item.group == group)
                .collect();
            (!items.is_empty()).then(|| group_list(group, &items, &palette))
        });
        div()
            .flex()
            .flex_col()
            .gap(px(14.))
            .pb(px(4.))
            .child(div().text_color(palette.text2).child(note))
            .child(
                div()
                    .id("storage-preview-list")
                    .max_h(px(360.))
                    .overflow_y_scroll()
                    .flex()
                    .flex_col()
                    .gap(px(12.))
                    .children(groups),
            )
            .child(
                div()
                    .flex()
                    .justify_end()
                    .gap(px(8.))
                    .child(cancel)
                    .child(remove),
            )
    }
}

fn group_list(group: ReclaimGroup, items: &[&ReclaimItem], palette: &Palette) -> Div {
    div()
        .flex()
        .flex_col()
        .gap(px(4.))
        .child(
            div()
                .text_size(px(11.))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(palette.text2)
                .child(group.label()),
        )
        .children(items.iter().map(|item| {
            div()
                .flex()
                .gap(px(12.))
                .text_size(px(12.))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .truncate()
                        .font_family(palette.mono())
                        .child(item.name.clone()),
                )
                .child(
                    div()
                        .text_color(palette.text2)
                        .child(bytes_label(item.size)),
                )
        }))
}

fn count(n: usize) -> String {
    if n == 1 {
        "1 item".into()
    } else {
        format!("{n} items")
    }
}

/// Opens the preview of the checked groups as they are now.
pub fn open(model: Entity<StorageModel>, window: &mut Window, cx: &mut App) {
    let (items, snapshot_first) = {
        let storage = model.read(cx);
        let items: Vec<ReclaimItem> = storage.plan().selected(&storage.checked).cloned().collect();
        (items, storage.snapshot_first && storage.can_snapshot(cx))
    };
    let title = format!("Remove {}?", count(items.len()));
    let dialog = cx.new(|_| PreviewDialog {
        model,
        items,
        snapshot_first,
    });
    window.open_dialog(cx, move |modal, _, _| {
        modal
            .title(title.clone())
            .w(px(520.))
            .overlay_closable(false)
            .child(dialog.clone())
    });
}
