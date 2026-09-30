//! Asks before an install or an update. It names the publisher, lists what the
//! extension runs, and says that its code runs on this computer. See ADR 0011,
//! "Trust".

use std::cmp::Ordering;

use captain_core::extension::{Backend, ExtensionCandidate, ExtensionUpdate};
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
    /// Set for an update: the installed extension and the new image.
    update: Option<ExtensionUpdate>,
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
        let mut rows = Vec::new();
        if let Some(update) = &self.update {
            let installed = &update.extension;
            rows.push((
                "Installed".into(),
                versioned(&installed.image, &installed.labels.version),
                palette.text,
            ));
            let older = update.version_order() == Some(Ordering::Less);
            let color = if older { palette.orange } else { palette.text };
            let mut new = versioned(&candidate.image, &candidate.labels.version);
            if older {
                new.push_str(", an older version");
            }
            rows.push(("New".into(), new, color));
        } else {
            rows.push(("Image".into(), candidate.image.clone(), palette.text));
        }
        rows.extend([(
            "Publisher".into(),
            labels.publisher().to_string(),
            publisher_color,
        )]);
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
        let label = if self.update.is_some() {
            "Update"
        } else {
            "Install"
        };
        let install = text_button(
            "extension-install-ok",
            label,
            ButtonTone::Accent,
            true,
            &palette,
            move |_, window, cx| {
                this.update(cx, |dialog, cx| {
                    let (candidate, update) = (dialog.candidate.clone(), dialog.update.clone());
                    dialog.model.update(cx, |model, cx| match update {
                        Some(update) => model.update(update, cx),
                        None => model.install(candidate, cx),
                    });
                })
                .ok();
                window.close_dialog(cx);
            },
        );
        let this = cx.entity().downgrade();
        let cancel = text_button(
            "extension-install-cancel",
            "Cancel",
            ButtonTone::Accent,
            true,
            &palette,
            move |_, window, cx| {
                // An image Captain pulled only to ask goes again.
                this.update(cx, |dialog, cx| {
                    let candidate = dialog.candidate.clone();
                    dialog
                        .model
                        .update(cx, |model, cx| model.discard(candidate, cx));
                })
                .ok();
                window.close_dialog(cx);
            },
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
                    .text_color(palette.warn_text)
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
    show(model, candidate, None, window, cx);
}

/// Asks before replacing an installed extension with a new image.
pub fn open_update(
    model: Entity<ExtensionsModel>,
    update: ExtensionUpdate,
    window: &mut Window,
    cx: &mut App,
) {
    show(model, update.candidate.clone(), Some(update), window, cx);
}

fn show(
    model: Entity<ExtensionsModel>,
    candidate: ExtensionCandidate,
    update: Option<ExtensionUpdate>,
    window: &mut Window,
    cx: &mut App,
) {
    let verb = if update.is_some() {
        "Update"
    } else {
        "Install"
    };
    let title = match candidate.labels.title.as_str() {
        "" => format!("{verb} {}?", candidate.image),
        name => format!("{verb} {name}?"),
    };
    let dialog = cx.new(|_| InstallDialog {
        model,
        candidate,
        update,
    });
    window.open_dialog(cx, move |modal, _, _| {
        modal
            .title(title.clone())
            .w(px(520.))
            .overlay_closable(false)
            // Cancel removes an image the check pulled; the close button would not.
            .close_button(false)
            .child(dialog.clone())
    });
}

/// `image (version 1.2.0)`, or the image alone when it names no version.
fn versioned(image: &str, version: &str) -> String {
    match version {
        "" => image.to_string(),
        version => format!("{image} (version {version})"),
    }
}
