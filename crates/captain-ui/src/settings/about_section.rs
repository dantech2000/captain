//! The About line at the bottom of Settings: Captain's version, the engine
//! runtime, the Docker version, and the licenses. The About Captain dialog shows
//! the same facts.

use captain_core::cli_tools::running_bundle;
use gpui_kit::*;

use super::SettingsView;
use crate::engine_host::HostModel;
use crate::help::HelpExt;
use crate::theme::Palette;
use crate::workspace::{Connection, Workspace};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");

/// "Captain 0.1.0", then the engine runtime and the Docker version when known.
pub fn versions(
    host: Option<&Entity<HostModel>>,
    workspace: &Entity<Workspace>,
    cx: &App,
) -> Vec<String> {
    let mut parts = vec![format!("Captain {VERSION}")];
    parts.extend(host.and_then(|host| host.read(cx).runtime_version()));
    if let Connection::Connected(info) = workspace.read(cx).connection() {
        parts.push(format!("Docker {}", info.version));
    }
    parts
}

/// Opens the licenses folder of the running app, or the repository when there is none.
pub fn open_licenses(cx: &mut App) {
    match running_bundle().map(|bundle| bundle.licenses()) {
        Some(dir) if dir.exists() => cx.open_with_system(&dir),
        _ => cx.open_url(REPOSITORY),
    }
}

pub fn render(view: &SettingsView, palette: &Palette, cx: &App) -> Div {
    let mut parts = versions(view.host.as_ref(), &view.workspace, cx);
    parts.push(String::new());
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
                .on_click(|_, _, cx| open_licenses(cx))
                .child("Licenses")
                .help("Show the licenses of Captain and the tools it ships."),
        )
}
