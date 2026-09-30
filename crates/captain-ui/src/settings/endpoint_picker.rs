use gpui_kit::component::Sizable;
use gpui_kit::component::input::{Input, InputState};
use gpui_kit::*;

use super::engine_source::DetectedEndpoint;
use super::{SettingsView, context_rows};
use crate::help::HelpExt;
use crate::theme::Palette;
use crate::widgets::{ButtonTone, settings_card, settings_row, text_button};
use crate::workspace::Connection;

/// The Switch engine card: the engines found on this machine, and a field for any
/// other endpoint.
pub fn render(
    view: &SettingsView,
    input: &Entity<InputState>,
    palette: &Palette,
    cx: &mut Context<SettingsView>,
) -> Div {
    let in_use = match view.workspace.read(cx).connection() {
        Connection::Connected(info) => Some(info.endpoint.clone()),
        _ => None,
    };
    let mut rows: Vec<AnyElement> = view
        .detected
        .iter()
        .enumerate()
        .map(|(ix, detected)| {
            let active = in_use.as_deref() == Some(detected.host.as_ref());
            detected_row(ix, detected, active, palette, cx)
        })
        .collect();
    // Contexts are engines too, so the empty note shows only when neither list has any.
    if rows.is_empty() && view.contexts.contexts.is_empty() {
        rows.push(
            settings_row(
                "No engines found",
                Some("Start an engine, then click Rescan.".into()),
                div(),
                palette,
            )
            .into_any_element(),
        );
    }
    rows.extend(context_rows::rows(view, in_use.as_deref(), palette, cx));
    rows.push(rescan_row(palette, cx));
    rows.push(custom_row(view, input, palette, cx));
    settings_card("Switch engine", rows, palette)
}

fn detected_row(
    ix: usize,
    detected: &DetectedEndpoint,
    active: bool,
    palette: &Palette,
    cx: &mut Context<SettingsView>,
) -> AnyElement {
    let host = detected.host.to_string();
    let label = if active { "In use" } else { "Use" };
    let button = text_button(
        ("engine-detected", ix),
        label,
        ButtonTone::Accent,
        !active,
        palette,
        cx.listener(move |view, _, _, cx| view.use_engine(Some(host.clone()), cx)),
    );
    settings_row(
        div()
            .font_family(palette.mono())
            .text_size(px(12.))
            .child(detected.host.clone()),
        Some(detected.source.clone()),
        button,
        palette,
    )
    .id(("settings-detected", ix))
    .help(format!("Connect to the engine at {}.", detected.host))
    .into_any_element()
}

fn rescan_row(palette: &Palette, cx: &mut Context<SettingsView>) -> AnyElement {
    settings_row(
        "Look again",
        Some("Captain checks DOCKER_HOST, the Docker contexts, and known sockets.".into()),
        text_button(
            "engine-rescan",
            "Rescan",
            ButtonTone::Accent,
            true,
            palette,
            cx.listener(|view, _, _, cx| view.rescan(cx)),
        ),
        palette,
    )
    .id("settings-rescan")
    .help("Look again for engines: DOCKER_HOST, the Docker contexts, and known sockets.")
    .into_any_element()
}

fn custom_row(
    view: &SettingsView,
    input: &Entity<InputState>,
    palette: &Palette,
    cx: &mut Context<SettingsView>,
) -> AnyElement {
    let field = div()
        .flex()
        .items_center()
        .gap(px(8.))
        .child(div().w(px(300.)).child(Input::new(input).small()))
        .child(text_button(
            "engine-custom",
            "Use this engine",
            ButtonTone::Accent,
            true,
            palette,
            cx.listener(|view, _, window, cx| view.use_custom(window, cx)),
        ));
    let control = div()
        .flex()
        .flex_col()
        .items_end()
        .gap(px(4.))
        .child(field)
        .children(
            view.hint
                .clone()
                .map(|hint| div().text_size(px(11.)).text_color(palette.red).child(hint)),
        );
    settings_row(
        "Custom endpoint",
        Some("A unix://, npipe://, tcp://, http://, or ssh://user@host URL.".into()),
        control,
        palette,
    )
    .id("settings-custom-endpoint")
    .help("Type the URL of an engine, then connect to it with Use this engine.")
    .into_any_element()
}
