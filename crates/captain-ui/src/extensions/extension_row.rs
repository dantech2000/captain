use captain_core::extension::InstalledExtension;
use gpui_kit::*;

use super::{ExtensionsModel, remove_dialog, update_dialog};
use crate::help::HelpExt;
use crate::theme::Palette;
use crate::widgets::{ButtonTone, text_button};

/// One extension: title, image and publisher, and Open, Update, and Remove. Open is off for
/// an extension without a page, where Captain has no web view, and while a step runs.
pub fn render(
    model: &Entity<ExtensionsModel>,
    extension: &InstalledExtension,
    enabled: bool,
    palette: &Palette,
) -> AnyElement {
    let details = format!("{} · {}", extension.image, extension.labels.publisher());
    let can_open = enabled && super::CAN_OPEN && extension.page_url().is_some();
    let title = extension.title().to_string();
    let open = {
        let (model, extension) = (model.clone(), extension.clone());
        text_button(
            SharedString::from(format!("extension-open-{}", extension.id)),
            "Open",
            ButtonTone::Accent,
            can_open,
            palette,
            move |_, _, cx| {
                if let Some(manager) = model.read(cx).manager(cx) {
                    super::open_window(extension.clone(), manager, cx);
                }
            },
        )
        .help(format!("Open {title} in its own window."))
    };
    let update = {
        let (model, extension) = (model.clone(), extension.clone());
        text_button(
            SharedString::from(format!("extension-update-{}", extension.id)),
            "Update…",
            ButtonTone::Accent,
            enabled,
            palette,
            move |_, window, cx| update_dialog::open(model.clone(), extension.clone(), window, cx),
        )
        .help(format!(
            "Check for a newer image of {title} and install it. Its data stays."
        ))
    };
    let remove = {
        let (model, extension) = (model.clone(), extension.clone());
        text_button(
            SharedString::from(format!("extension-remove-{}", extension.id)),
            "Remove…",
            ButtonTone::Danger,
            enabled,
            palette,
            move |_, window, cx| remove_dialog::open(model.clone(), extension.clone(), window, cx),
        )
        .help(format!(
            "Remove {title} and its backend. Captain asks first."
        ))
    };
    div()
        .min_h(px(52.))
        .px(px(14.))
        .py(px(8.))
        .flex()
        .items_center()
        .gap(px(16.))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .gap(px(2.))
                .child(
                    div()
                        .font_weight(FontWeight::MEDIUM)
                        .truncate()
                        .child(extension.title().to_string()),
                )
                .children((!extension.labels.description.is_empty()).then(|| {
                    div()
                        .text_size(px(12.))
                        .text_color(palette.text2)
                        .truncate()
                        .child(extension.labels.description.clone())
                }))
                .child(
                    div()
                        .text_size(px(11.))
                        .text_color(palette.text3)
                        .truncate()
                        .child(details),
                ),
        )
        .child(open)
        .child(update)
        .child(remove)
        .into_any_element()
}
