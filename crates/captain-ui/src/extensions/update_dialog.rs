//! The Update dialog: the tag to pull, `latest` by default. Check pulls it and
//! compares it with the installed image.

use captain_core::extension::{DEFAULT_UPDATE_TAG, InstalledExtension};
use gpui_kit::component::Sizable;
use gpui_kit::component::WindowExt;
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::*;

use super::ExtensionsModel;
use crate::images::field;
use crate::theme::Palette;
use crate::widgets::{ButtonTone, text_button};

struct UpdateDialog {
    model: Entity<ExtensionsModel>,
    extension: InstalledExtension,
    tag: Entity<InputState>,
    _subscription: Subscription,
}

impl UpdateDialog {
    fn new(
        model: Entity<ExtensionsModel>,
        extension: InstalledExtension,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let tag = cx.new(|cx| InputState::new(window, cx).default_value(DEFAULT_UPDATE_TAG));
        let subscription = cx.subscribe_in(&tag, window, |this, _, event, window, cx| {
            if let InputEvent::PressEnter { .. } = event {
                this.submit(window, cx);
            }
        });
        Self {
            model,
            extension,
            tag,
            _subscription: subscription,
        }
    }

    fn submit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let tag = self.tag.read(cx).value().trim().to_string();
        let extension = self.extension.clone();
        self.model
            .update(cx, |model, cx| model.check_update(extension, tag, cx));
        window.close_dialog(cx);
    }
}

impl Render for UpdateDialog {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = Palette::of(cx);
        let this = cx.entity().downgrade();
        let check = text_button(
            "extension-update-check",
            "Check",
            ButtonTone::Accent,
            true,
            &palette,
            move |_, window, cx| {
                this.update(cx, |this, cx| this.submit(window, cx)).ok();
            },
        );
        let cancel = text_button(
            "extension-update-cancel",
            "Cancel",
            ButtonTone::Accent,
            true,
            &palette,
            |_, window, cx| window.close_dialog(cx),
        );
        let note = format!(
            "Captain pulls this tag of the extension's repository and compares it with \
             the installed image, {}. The update keeps the backend's volumes.",
            self.extension.image
        );
        div()
            .flex()
            .flex_col()
            .gap(px(16.))
            .pb(px(4.))
            .child(field(
                "Tag",
                Input::new(&self.tag).small().into_any_element(),
                &palette,
            ))
            .child(
                div()
                    .text_size(px(12.))
                    .text_color(palette.text2)
                    .child(note),
            )
            .child(
                div()
                    .flex()
                    .justify_end()
                    .gap(px(8.))
                    .child(cancel)
                    .child(check),
            )
    }
}

/// Opens the Update dialog for `extension`.
pub fn open(
    model: Entity<ExtensionsModel>,
    extension: InstalledExtension,
    window: &mut Window,
    cx: &mut App,
) {
    let title = format!("Update {}", extension.title());
    let form = cx.new(|cx| UpdateDialog::new(model, extension, window, cx));
    window.open_dialog(cx, move |dialog, _, _| {
        dialog
            .title(title.clone())
            .w(px(460.))
            .overlay_closable(false)
            .child(form.clone())
    });
}
