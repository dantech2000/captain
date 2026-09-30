//! The Update dialog: the tag to pull, the newest version tag of the repository by
//! default. Check pulls it and compares it with the installed image.

use captain_core::extension::{InstalledExtension, tag_version};
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
    /// The newest tag of the repository, once the lookup answers.
    newest: Option<String>,
    _lookup: Option<Task<()>>,
    _subscription: Subscription,
}

impl UpdateDialog {
    fn new(
        model: Entity<ExtensionsModel>,
        extension: InstalledExtension,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let tag = cx.new(|cx| InputState::new(window, cx).placeholder("Newest version"));
        let lookup = model.read(cx).manager(cx).map(|manager| {
            let newest = manager.newest_tag(extension.clone());
            cx.spawn_in(window, async move |this, cx| {
                let Ok(newest) = newest.await else {
                    return;
                };
                this.update_in(cx, |this, window, cx| {
                    // Keep a tag the user typed meanwhile.
                    if this.tag.read(cx).value().trim().is_empty() {
                        this.tag
                            .update(cx, |input, cx| input.set_value(newest.clone(), window, cx));
                    }
                    this.newest = Some(newest);
                    cx.notify();
                })
                .ok();
            })
        });
        let subscription = cx.subscribe_in(&tag, window, |this, _, event, window, cx| {
            if let InputEvent::PressEnter { .. } = event {
                this.submit(window, cx);
            }
        });
        Self {
            model,
            extension,
            tag,
            newest: None,
            _lookup: lookup,
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
        let installed = installed_tag(&self.extension.image);
        let found = match (&self.newest, installed) {
            (None, _) => "Looking up the newest version tag...".to_string(),
            (Some(newest), Some(installed)) if !is_newer(newest, installed) => format!(
                "No version newer than {installed} is published. Check pulls {newest} \
                 and compares it anyway."
            ),
            (Some(newest), _) => format!("The newest published version is {newest}."),
        };
        let note = format!(
            "Installed: {}. {found} The update keeps the backend's volumes.",
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

/// The tag of an image reference, if it names one.
fn installed_tag(image: &str) -> Option<&str> {
    let last = image.rsplit('/').next()?;
    last.split_once(':').map(|(_, tag)| tag)
}

/// Whether `newest` is a later version than `installed`. Tags that are not
/// versions count as newer unless they are the same tag.
fn is_newer(newest: &str, installed: &str) -> bool {
    match (tag_version(newest), tag_version(installed)) {
        (Some(newest), Some(installed)) => newest > installed,
        _ => newest != installed,
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
