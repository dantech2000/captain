//! One row per shell file in the Command-line tools card: what it has, and the
//! line to add with a Copy button when Captain does not write it. See feature 0035.

use captain_core::cli_tools::{PathMode, RcAccess, RcState, RcStatus, ToolPaths};
use captain_core::settings::Settings;
use gpui_kit::component::WindowExt;
use gpui_kit::component::notification::Notification;
use gpui_kit::*;

use super::SettingsView;
use crate::help::HelpExt;
use crate::theme::Palette;
use crate::widgets::{ButtonTone, settings_row, text_button};

pub fn rows(
    view: &SettingsView,
    settings: &Settings,
    paths: Option<&ToolPaths>,
    palette: &Palette,
) -> Vec<AnyElement> {
    let mode = settings.command_line_tools.path;
    let Some(status) = &view.cli_tools.status else {
        return Vec::new();
    };
    status
        .rc
        .iter()
        .enumerate()
        .map(|(ix, rc)| row(ix, rc, mode, paths, palette))
        .collect()
}

fn row(
    ix: usize,
    rc: &RcStatus,
    mode: PathMode,
    paths: Option<&ToolPaths>,
    palette: &Palette,
) -> AnyElement {
    let name = paths.map_or_else(
        || rc.file.path.display().to_string(),
        |paths| paths.tilde(&rc.file.path),
    );
    let line = rc.file.shell.path_line();
    let note = match (rc.state, &rc.access) {
        (RcState::Added, _) => "Has Captain's lines.".to_string(),
        (RcState::Present, _) => "Adds ~/.captain/bin to PATH.".into(),
        (RcState::Missing, RcAccess::Skip(why)) => format!("{why} Add this line yourself: {line}"),
        (RcState::Missing, RcAccess::Writable) if mode == PathMode::Manual => {
            format!("Add this line: {line}")
        }
        (RcState::Missing, RcAccess::Writable) => "Relink adds Captain's lines.".into(),
    };
    let copy = rc.needs_user(mode).then(|| {
        text_button(
            ("settings-tools-copy", ix),
            "Copy",
            ButtonTone::Accent,
            true,
            palette,
            move |_, window, cx| {
                cx.write_to_clipboard(ClipboardItem::new_string(line.to_string()));
                let message = "Copied the PATH line. Add it to the file, then open a new terminal.";
                window.push_notification(Notification::success(message), cx);
            },
        )
        .help(format!(
            "Copy the line that puts ~/.captain/bin on PATH for {}.",
            rc.file.shell.name()
        ))
    });
    settings_row(
        div()
            .font_family(palette.mono())
            .text_size(px(12.))
            .child(name.clone()),
        Some(note.into()),
        div().children(copy),
        palette,
    )
    .id(("settings-tools-rc", ix))
    .help(format!("Whether {name} puts ~/.captain/bin on PATH."))
    .into_any_element()
}
