//! The Edit dialog: a new name and description for a snapshot.

use captain_core::snapshot::{Snapshot, check_name};
use gpui_kit::component::Sizable;
use gpui_kit::component::WindowExt;
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::*;

use super::SnapshotsModel;
use crate::images::field;
use crate::theme::Palette;
use crate::widgets::{ButtonTone, inline_error, text_button};

struct EditDialog {
    model: Entity<SnapshotsModel>,
    snapshot: Snapshot,
    name: Entity<InputState>,
    description: Entity<InputState>,
    error: Option<String>,
    _subscriptions: Vec<Subscription>,
}

impl EditDialog {
    fn new(
        model: Entity<SnapshotsModel>,
        snapshot: Snapshot,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let metadata = &snapshot.metadata;
        let name = cx.new(|cx| InputState::new(window, cx).default_value(metadata.name.clone()));
        let description = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Optional")
                .default_value(metadata.description.clone())
        });
        let subscriptions = [&name, &description]
            .into_iter()
            .map(|input| {
                cx.subscribe_in(input, window, |this, _, event, window, cx| {
                    if let InputEvent::PressEnter { .. } = event {
                        this.submit(window, cx);
                    }
                })
            })
            .collect();
        Self {
            model,
            snapshot,
            name,
            description,
            error: None,
            _subscriptions: subscriptions,
        }
    }

    fn submit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let name = self.name.read(cx).value().to_string();
        let description = self.description.read(cx).value().to_string();
        let id = &self.snapshot.id;
        let model = self.model.read(cx);
        let others = model.list().snapshots.iter().filter(|s| &s.id != id);
        if let Err(error) = check_name(&name, others.map(|s| s.metadata.name.as_str())) {
            self.error = Some(error);
            cx.notify();
            return;
        }
        let snapshot = self.snapshot.clone();
        self.model
            .update(cx, |model, cx| model.edit(snapshot, name, description, cx));
        window.close_dialog(cx);
    }
}

impl Render for EditDialog {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = Palette::of(cx);
        let this = cx.entity().downgrade();
        let save = text_button(
            "snapshot-edit-ok",
            "Save",
            ButtonTone::Accent,
            true,
            &palette,
            move |_, window, cx| {
                this.update(cx, |this, cx| this.submit(window, cx)).ok();
            },
        );
        let cancel = text_button(
            "snapshot-edit-cancel",
            "Cancel",
            ButtonTone::Accent,
            true,
            &palette,
            |_, window, cx| window.close_dialog(cx),
        );
        div()
            .flex()
            .flex_col()
            .gap(px(16.))
            .pb(px(4.))
            .child(field(
                "Name",
                Input::new(&self.name).small().into_any_element(),
                &palette,
            ))
            .child(field(
                "Description",
                Input::new(&self.description).small().into_any_element(),
                &palette,
            ))
            .children(self.error.clone().map(|e| inline_error(e, &palette)))
            .child(
                div()
                    .flex()
                    .justify_end()
                    .gap(px(8.))
                    .child(cancel)
                    .child(save),
            )
    }
}

/// Opens the Edit dialog for `snapshot`.
pub fn open(model: Entity<SnapshotsModel>, snapshot: Snapshot, window: &mut Window, cx: &mut App) {
    let form = cx.new(|cx| EditDialog::new(model, snapshot, window, cx));
    window.open_dialog(cx, move |dialog, _, _| {
        dialog
            .title("Edit snapshot")
            .w(px(460.))
            .overlay_closable(false)
            .child(form.clone())
    });
}
