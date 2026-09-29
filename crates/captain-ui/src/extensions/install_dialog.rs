//! Asks before an install. It names the publisher, lists what the extension runs,
//! and says that its code runs on this computer. See ADR 0011, "Trust".

use captain_core::extension::{Backend, ExtensionCandidate};
use gpui_kit::component::WindowExt;
use gpui_kit::*;

use super::ExtensionsModel;
use crate::theme::Palette;
use crate::widgets::{ButtonTone, colored_key_values, text_button};

/// The warning on the page and in this dialog.
pub const TRUST_NOTE: &str = "Extensions run code from their publisher on this computer and \
    in the engine, with your permissions. Install only extensions you trust.";

struct InstallDialog {
    model: Entity<ExtensionsModel>,
    candidate: ExtensionCandidate,
}

impl InstallDialog {
    /// The rows: image, publisher, and each part the extension runs.
    fn rows(&self, palette: &Palette) -> Vec<(String, String, Hsla)> {
        let candidate = &self.candidate;
        let labels = &candidate.labels;
        let publisher_color = if labels.vendor.is_empty() {
            palette.orange
        } else {
            palette.text
        };
        let mut rows = vec![
            ("Image".into(), candidate.image.clone(), palette.text),
            (
                "Publisher".into(),
                labels.publisher().to_string(),
                publisher_color,
            ),
        ];
        if !labels.publisher_url.is_empty() {
            rows.push(("Website".into(), labels.publisher_url.clone(), palette.text));
        }
        if candidate.metadata.dashboard_tab().is_some() {
            rows.push((
                "Page".into(),
                "Opens in its own window".into(),
                palette.text,
            ));
        }
        let backend = match candidate.metadata.backend(&candidate.image) {
            Some(Backend::Image(image)) => Some(format!("Runs {image} in the engine")),
            Some(Backend::Compose(file)) => {
                Some(format!("Runs the Compose file {file} in the engine"))
            }
            None => None,
        };
        rows.extend(backend.map(|text| ("Backend".into(), text, palette.text)));
        let binaries = candidate.binary_names();
        if !binaries.is_empty() {
            rows.push(("Host binaries".into(), binaries.join(", "), palette.orange));
        }
        rows
    }
}

impl Render for InstallDialog {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = Palette::of(cx);
        let this = cx.entity().downgrade();
        let install = text_button(
            "extension-install-ok",
            "Install",
            ButtonTone::Accent,
            true,
            &palette,
            move |_, window, cx| {
                this.update(cx, |dialog, cx| {
                    let candidate = dialog.candidate.clone();
                    dialog
                        .model
                        .update(cx, |model, cx| model.install(candidate, cx));
                })
                .ok();
                window.close_dialog(cx);
            },
        );
        let cancel = text_button(
            "extension-install-cancel",
            "Cancel",
            ButtonTone::Accent,
            true,
            &palette,
            |_, window, cx| window.close_dialog(cx),
        );
        let description = self.candidate.labels.description.clone();
        div()
            .flex()
            .flex_col()
            .gap(px(14.))
            .pb(px(4.))
            .children(
                (!description.is_empty())
                    .then(|| div().text_color(palette.text2).child(description)),
            )
            .child(colored_key_values(self.rows(&palette), &palette))
            .child(
                div()
                    .text_size(px(12.))
                    .text_color(palette.orange)
                    .child(TRUST_NOTE),
            )
            .child(
                div()
                    .flex()
                    .justify_end()
                    .gap(px(8.))
                    .child(cancel)
                    .child(install),
            )
    }
}

pub fn open(
    model: Entity<ExtensionsModel>,
    candidate: ExtensionCandidate,
    window: &mut Window,
    cx: &mut App,
) {
    let title = match candidate.labels.title.as_str() {
        "" => format!("Install {}?", candidate.image),
        name => format!("Install {name}?"),
    };
    let dialog = cx.new(|_| InstallDialog { model, candidate });
    window.open_dialog(cx, move |modal, _, _| {
        modal
            .title(title.clone())
            .w(px(520.))
            .overlay_closable(false)
            .child(dialog.clone())
    });
}
