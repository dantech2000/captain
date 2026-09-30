//! The About line at the bottom of Settings: Captain's version, the engine
//! runtime, the Docker version, and the licenses.

use captain_core::cli_tools::running_bundle;
use gpui_kit::*;

use super::SettingsView;
use crate::help::HelpExt;
use crate::theme::Palette;
use crate::workspace::Connection;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");

pub fn render(view: &SettingsView, palette: &Palette, cx: &App) -> Div {
    let mut parts = vec![format!("Captain {VERSION}")];
    parts.extend(
        view.host
            .as_ref()
            .and_then(|host| host.read(cx).runtime_version()),
    );
    if let Connection::Connected(info) = view.workspace.read(cx).connection() {
        parts.push(format!("Docker {}", info.version));
    }
    parts.push(String::new());
    let licenses = running_bundle().map(|bundle| bundle.licenses());
    div()
        .flex()
        .justify_center()
        .items_center()
        .pt(px(6.))
        .text_size(px(11.5))
        .text_color(palette.text3)
        .child(parts.join(" \u{00b7} "))
        .child(
            div()
                .id("settings-licenses")
                .text_color(palette.text2)
                .underline()
                .cursor_pointer()
                .on_click(move |_, _, cx| match &licenses {
                    Some(dir) if dir.exists() => cx.open_with_system(dir),
                    _ => cx.open_url(REPOSITORY),
                })
                .child("Licenses")
                .help("Show the licenses of Captain and the tools it ships."),
        )
}
