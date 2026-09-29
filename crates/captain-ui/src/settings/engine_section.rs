use captain_core::format::bytes_label;
use captain_core::model::EngineInfo;
use captain_core::settings::Settings;
use gpui_kit::*;

use super::SettingsView;
use super::engine_source;
use crate::theme::Palette;
use crate::widgets::{ButtonTone, pill, settings_card, settings_row, text_button};
use crate::workspace::Connection;

/// The Connection card: the connection state, what the engine reports, and the endpoint
/// Captain uses.
pub fn render(
    view: &SettingsView,
    settings: &Settings,
    palette: &Palette,
    cx: &mut Context<SettingsView>,
) -> Div {
    let connection = view.workspace.read(cx).connection().clone();
    let workspace = view.workspace.clone();
    let reconnect = text_button(
        "engine-reconnect",
        "Reconnect",
        ButtonTone::Accent,
        !matches!(connection, Connection::Connecting),
        palette,
        move |_, _, cx| engine_source::reconnect(&workspace, cx),
    );

    let (state, color, note) = match &connection {
        Connection::Connecting => ("Connecting", palette.orange, "Connecting...".into()),
        Connection::Connected(info) => ("Connected", palette.green, info.endpoint.clone()),
        Connection::Failed(error) => ("Not connected", palette.red, error.to_string()),
    };
    let mut rows = vec![
        settings_row(
            "Status",
            Some(note.into()),
            div()
                .flex()
                .items_center()
                .gap(px(8.))
                .child(pill(state, color, palette.tint(color)))
                .child(reconnect),
            palette,
        )
        .into_any_element(),
    ];
    if let Connection::Connected(info) = &connection {
        rows.extend(info_rows(info, palette));
    }
    rows.push(endpoint_row(settings, palette, cx));

    settings_card("Connection", rows, palette)
}

/// What the engine reports about itself.
fn info_rows(info: &EngineInfo, palette: &Palette) -> Vec<AnyElement> {
    let value = |text: String| div().text_color(palette.text2).child(text);
    vec![
        settings_row(
            "Version",
            None,
            value(format!("{} (API {})", info.version, info.api_version)),
            palette,
        )
        .into_any_element(),
        settings_row(
            "Platform",
            None,
            value(format!("{} / {}", info.os, info.arch)),
            palette,
        )
        .into_any_element(),
        settings_row(
            "Resources",
            None,
            value(format!(
                "{} CPUs · {} memory",
                info.cpus,
                bytes_label(info.memory_bytes)
            )),
            palette,
        )
        .into_any_element(),
    ]
}

/// The saved endpoint, or discovery, with a way back to discovery.
fn endpoint_row(
    settings: &Settings,
    palette: &Palette,
    cx: &mut Context<SettingsView>,
) -> AnyElement {
    let Some(endpoint) = settings.engine_endpoint.clone() else {
        return settings_row(
            "Endpoint",
            Some(
                "Captain finds the engine: DOCKER_HOST, the current context, then known sockets."
                    .into(),
            ),
            div().text_color(palette.text2).child("Automatic"),
            palette,
        )
        .into_any_element();
    };
    let automatic = text_button(
        "engine-automatic",
        "Use automatic",
        ButtonTone::Accent,
        true,
        palette,
        cx.listener(|view, _, _, cx| view.use_engine(None, cx)),
    );
    settings_row(
        "Endpoint",
        Some(endpoint.into()),
        div()
            .flex()
            .items_center()
            .gap(px(8.))
            .child(div().text_color(palette.text2).child("Custom"))
            .child(automatic),
        palette,
    )
    .into_any_element()
}
