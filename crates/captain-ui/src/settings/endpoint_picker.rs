//! The engines sheet's content: the endpoint Captain uses, the engines found on
//! this machine, the Docker contexts, and a field for any other endpoint.

use captain_core::settings::Settings;
use gpui_kit::component::Sizable;
use gpui_kit::component::input::{Input, InputState};
use gpui_kit::*;

use super::engine_source::DetectedEndpoint;
use super::listen::listen;
use super::{SettingsView, context_rows};
use crate::help::HelpExt;
use crate::theme::Palette;
use crate::widgets::{ButtonTone, settings_card, settings_row, text_button};
use crate::workspace::Connection;

pub fn render(
    view: &SettingsView,
    this: &WeakEntity<SettingsView>,
    settings: &Settings,
    input: Option<&Entity<InputState>>,
    palette: &Palette,
    cx: &App,
) -> Div {
    let in_use = match view.workspace.read(cx).connection() {
        Connection::Connected(info) => Some(info.endpoint.clone()),
        _ => None,
    };
    let mut rows: Vec<AnyElement> = vec![endpoint_row(settings, this, palette)];
    rows.extend(view.detected.iter().enumerate().map(|(ix, detected)| {
        let active = in_use.as_deref() == Some(detected.host.as_ref());
        detected_row(ix, detected, active, this, palette)
    }));
    // Contexts are engines too, so the empty note shows only when neither list has any.
    if view.detected.is_empty() && view.contexts.contexts.is_empty() {
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
    rows.extend(context_rows::rows(
        view,
        this,
        in_use.as_deref(),
        palette,
        cx,
    ));
    rows.push(rescan_row(this, palette));
    rows.extend(input.map(|input| custom_row(view, this, input, palette)));
    settings_card("Engines", rows, palette)
}

/// The saved endpoint, or discovery, with a way back to discovery.
fn endpoint_row(
    settings: &Settings,
    this: &WeakEntity<SettingsView>,
    palette: &Palette,
) -> AnyElement {
    let Some(endpoint) = settings.engine_endpoint.clone() else {
        return settings_row(
            "Endpoint: Automatic",
            Some(
                "Captain finds the engine: DOCKER_HOST, the current context, then known sockets."
                    .into(),
            ),
            div(),
            palette,
        )
        .into_any_element();
    };
    settings_row(
        "Endpoint: Custom",
        Some(endpoint.into()),
        text_button(
            "engine-automatic",
            "Use automatic",
            ButtonTone::Accent,
            true,
            palette,
            listen(this, |view, _, cx| view.use_engine(None, cx)),
        ),
        palette,
    )
    .id("settings-endpoint")
    .help("Go back to finding the engine automatically.")
    .into_any_element()
}

fn detected_row(
    ix: usize,
    detected: &DetectedEndpoint,
    active: bool,
    this: &WeakEntity<SettingsView>,
    palette: &Palette,
) -> AnyElement {
    let host = detected.host.to_string();
    let label = if active { "In use" } else { "Use" };
    let button = text_button(
        ("engine-detected", ix),
        label,
        ButtonTone::Accent,
        !active,
        palette,
        listen(this, move |view, _, cx| {
            view.use_engine(Some(host.clone()), cx)
        }),
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

fn rescan_row(this: &WeakEntity<SettingsView>, palette: &Palette) -> AnyElement {
    settings_row(
        "Look again",
        Some("Captain checks DOCKER_HOST, the Docker contexts, and known sockets.".into()),
        text_button(
            "engine-rescan",
            "Rescan",
            ButtonTone::Accent,
            true,
            palette,
            listen(this, |view, _, cx| view.rescan(cx)),
        ),
        palette,
    )
    .id("settings-rescan")
    .help("Look again for engines: DOCKER_HOST, the Docker contexts, and known sockets.")
    .into_any_element()
}

fn custom_row(
    view: &SettingsView,
    this: &WeakEntity<SettingsView>,
    input: &Entity<InputState>,
    palette: &Palette,
) -> AnyElement {
    let field = div()
        .flex()
        .items_center()
        .gap(px(8.))
        .child(div().w(px(260.)).child(Input::new(input).small()))
        .child(text_button(
            "engine-custom",
            "Connect",
            ButtonTone::Accent,
            true,
            palette,
            listen(this, |view, window, cx| view.use_custom(window, cx)),
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
        "Remote host",
        Some("ssh://user@host, or a unix://, npipe://, tcp://, or http:// URL.".into()),
        control,
        palette,
    )
    .id("settings-custom-endpoint")
    .help("Type the URL of an engine, such as ssh://user@host, then connect to it.")
    .into_any_element()
}
