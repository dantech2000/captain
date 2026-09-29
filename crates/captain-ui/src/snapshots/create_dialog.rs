//! The Create dialog: a name, the date and time by default, and a description.

use captain_core::snapshot::{check_name, space_warning};
use gpui_kit::component::Sizable;
use gpui_kit::component::WindowExt;
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::*;

use super::SnapshotsModel;
use crate::images::field;
use crate::theme::Palette;
use crate::widgets::{ButtonTone, inline_error, text_button};

struct CreateDialog {
    model: Entity<SnapshotsModel>,
    name: Entity<InputState>,
    description: Entity<InputState>,
    /// The engine runs, so the dialog says it stops meanwhile.
    running: bool,
    /// Less space is free than the engine's disk uses.
    warning: Option<String>,
    error: Option<String>,
    _subscriptions: Vec<Subscription>,
}

impl CreateDialog {
    fn new(
        model: Entity<SnapshotsModel>,
        running: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let default = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();
        let name = cx.new(|cx| InputState::new(window, cx).default_value(default));
        let description = cx.new(|cx| InputState::new(window, cx).placeholder("Optional"));
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
        let list = model.read(cx).list();
        let warning = space_warning(list.free_bytes, list.engine_disk_bytes);
        Self {
            model,
            name,
            description,
            running,
            warning,
            error: None,
            _subscriptions: subscriptions,
        }
    }

    fn submit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let name = self.name.read(cx).value().to_string();
        let description = self.description.read(cx).value().to_string();
        let model = self.model.read(cx);
        let names = model
            .list()
            .snapshots
            .iter()
            .map(|s| s.metadata.name.as_str());
        if let Err(error) = check_name(&name, names) {
            self.error = Some(error);
            cx.notify();
            return;
        }
        self.model
            .update(cx, |model, cx| model.create(name, description, cx));
        window.close_dialog(cx);
    }
}

impl Render for CreateDialog {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = Palette::of(cx);
        let this = cx.entity().downgrade();
        let create = text_button(
            "snapshot-create-ok",
            "Create",
            ButtonTone::Accent,
            true,
            &palette,
            move |_, window, cx| {
                this.update(cx, |this, cx| this.submit(window, cx)).ok();
            },
        );
        let cancel = text_button(
            "snapshot-create-cancel",
            "Cancel",
            ButtonTone::Accent,
            true,
            &palette,
            |_, window, cx| window.close_dialog(cx),
        );
        let note = if self.running {
            "Captain Engine stops while Captain saves the snapshot, then starts again."
        } else {
            "The snapshot holds the engine's disk: images, containers, and volumes."
        };
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
            .child(
                div()
                    .text_size(px(12.))
                    .text_color(palette.text2)
                    .child(note),
            )
            .children(self.warning.clone().map(|warning| {
                div()
                    .text_size(px(12.))
                    .text_color(palette.orange)
                    .child(warning)
            }))
            .children(self.error.clone().map(|e| inline_error(e, &palette)))
            .child(
                div()
                    .flex()
                    .justify_end()
                    .gap(px(8.))
                    .child(cancel)
                    .child(create),
            )
    }
}

/// Opens the Create dialog. `running` says whether the engine runs now.
pub fn open(model: Entity<SnapshotsModel>, running: bool, window: &mut Window, cx: &mut App) {
    let form = cx.new(|cx| CreateDialog::new(model, running, window, cx));
    window.open_dialog(cx, move |dialog, _, _| {
        dialog
            .title("Create snapshot")
            .w(px(460.))
            .overlay_closable(false)
            .child(form.clone())
    });
}
