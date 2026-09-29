use std::path::PathBuf;

use gpui_kit::component::switch::Switch;
use gpui_kit::*;

use super::DiagnosticsModel;
use crate::settings;
use crate::theme::Palette;
use crate::widgets::{ButtonTone, settings_card, settings_row, text_button};

/// Show logs, Show engine files, and the debug logging switch.
pub fn render(model: &DiagnosticsModel, debug_logging: bool, palette: &Palette) -> Div {
    let setup = &model.setup;
    let set_debug = setup.set_debug_logging.clone();
    let rows = [
        folder_row(
            "Logs",
            "Captain's own log files.",
            "Show logs",
            setup.log_dir.clone(),
            palette,
        ),
        folder_row(
            "Engine files",
            "Captain Engine's Lima instance folder, with Lima's logs.",
            "Show engine files",
            setup.engine_dir.clone().filter(|dir| dir.is_dir()),
            palette,
        ),
        settings_row(
            "Debug logging",
            Some("Writes more detail to the logs. Turn it on to report a problem.".into()),
            Switch::new("debug-logging")
                .checked(debug_logging)
                .on_click(move |checked, _, cx| {
                    let checked = *checked;
                    settings::update(cx, |settings| settings.debug_logging = checked);
                    set_debug(checked);
                }),
            palette,
        )
        .into_any_element(),
    ];
    settings_card("Troubleshooting", rows, palette)
}

/// A row with a button that opens `dir` in the file manager. Without a folder the
/// button is off.
fn folder_row(
    label: &'static str,
    note: &'static str,
    button: &'static str,
    dir: Option<PathBuf>,
    palette: &Palette,
) -> AnyElement {
    let note = match &dir {
        Some(dir) => format!("{note} {}", dir.display()),
        None => format!("{note} The folder does not exist yet."),
    };
    let enabled = dir.is_some();
    settings_row(
        label,
        Some(note.into()),
        text_button(
            button,
            button,
            ButtonTone::Accent,
            enabled,
            palette,
            move |_, _, cx| {
                if let Some(dir) = &dir {
                    cx.open_with_system(dir);
                }
            },
        ),
        palette,
    )
    .into_any_element()
}
