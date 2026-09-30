//! Asks before a restore, with the choice to save the current state first.

use captain_core::snapshot::Snapshot;
use gpui_kit::component::WindowExt;
use gpui_kit::component::checkbox::Checkbox;
use gpui_kit::*;

use super::SnapshotsModel;
use crate::help::HelpExt;
use crate::theme::Palette;
use crate::widgets::{ButtonTone, text_button};

struct RestoreDialog {
    model: Entity<SnapshotsModel>,
    snapshot: Snapshot,
    save_first: bool,
}

impl Render for RestoreDialog {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = Palette::of(cx);
        let this = cx.entity().downgrade();
        let toggle = this.clone();
        let restore = text_button(
            "snapshot-restore-ok",
            "Restore",
            ButtonTone::Danger,
            true,
            &palette,
            move |_, window, cx| {
                this.update(cx, |dialog, cx| {
                    let snapshot = dialog.snapshot.clone();
                    let save_first = dialog.save_first;
                    dialog
                        .model
                        .update(cx, |model, cx| model.restore(snapshot, save_first, cx));
                })
                .ok();
                window.close_dialog(cx);
            },
        )
        .help("Replace the engine's state with this snapshot. Captain Engine restarts.");
        let cancel = text_button(
            "snapshot-restore-cancel",
            "Cancel",
            ButtonTone::Accent,
            true,
            &palette,
            |_, window, cx| window.close_dialog(cx),
        )
        .help("Close this dialog. Nothing changes.");
        div()
            .flex()
            .flex_col()
            .gap(px(14.))
            .pb(px(4.))
            .child(div().text_color(palette.text2).child(
                "The current engine state is replaced by this snapshot. Containers, \
                 images, and volumes made since then are lost. Captain Engine restarts.",
            ))
            .child(
                div()
                    .id("snapshot-save-first-row")
                    .child(
                        Checkbox::new("snapshot-save-first")
                            .label("Save the current state as a snapshot first")
                            .checked(self.save_first)
                            .on_click(move |checked, _, cx| {
                                toggle
                                    .update(cx, |dialog, cx| {
                                        dialog.save_first = *checked;
                                        cx.notify();
                                    })
                                    .ok();
                            }),
                    )
                    .help("Save the current state as a snapshot first, so you can go back to it."),
            )
            .child(
                div()
                    .flex()
                    .justify_end()
                    .gap(px(8.))
                    .child(cancel)
                    .child(restore),
            )
    }
}

pub fn open(model: Entity<SnapshotsModel>, snapshot: Snapshot, window: &mut Window, cx: &mut App) {
    let title = format!("Restore \"{}\"?", snapshot.metadata.name);
    let dialog = cx.new(|_| RestoreDialog {
        model,
        snapshot,
        save_first: true,
    });
    window.open_dialog(cx, move |modal, _, _| {
        modal
            .title(title.clone())
            .w(px(460.))
            .overlay_closable(false)
            .child(dialog.clone())
    });
}
