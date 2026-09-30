//! The Engine section: the engine menu, "…", and Captain Engine's resources, or
//! the connection to another engine. See feature 0037.

use captain_core::HostStatus;
use captain_core::settings::EngineChoice;
use gpui_kit::*;

use super::page_section::{row, section, sub_row};
use super::{SettingsView, engine_menu, engine_resources, engine_source};
use crate::help::HelpExt;
use crate::theme::Palette;
use crate::widgets::{ButtonTone, text_button};
use crate::workspace::Connection;

pub fn render(view: &SettingsView, palette: &Palette, cx: &mut Context<SettingsView>) -> Div {
    let head = row("Engine", palette)
        .child(engine_menu::engine_button(view, palette, cx))
        .child(div().flex_1())
        .child(engine_menu::more_button(view, palette, cx));
    let captain = view.host.as_ref().filter(|host| {
        let host = host.read(cx);
        host.choice(cx) == EngineChoice::Captain && host.can_control()
    });
    let section = section(palette).child(head);
    match captain {
        Some(model) => {
            let host = model.read(cx);
            let problem = match host.status() {
                HostStatus::NotInstalled(why) | HostStatus::Failed(why) => Some(why.clone()),
                _ => None,
            };
            section
                .children(engine_resources::rows(model, host, view.free_disk, palette))
                .children(problem.map(|why| {
                    super::page_section::under_note(why, palette).text_color(palette.red)
                }))
        }
        None => section.child(connection_row(view, palette, cx)),
    }
}

/// The connection to another engine, and Reconnect.
fn connection_row(view: &SettingsView, palette: &Palette, cx: &App) -> Stateful<Div> {
    let connection = view.workspace.read(cx).connection().clone();
    let note = match &connection {
        Connection::Connecting => "Connecting\u{2026}".to_string(),
        Connection::Connected(info) => {
            format!("{} \u{00b7} Docker {}", info.endpoint, info.version)
        }
        Connection::Failed(error) => error.to_string(),
    };
    let workspace = view.workspace.clone();
    sub_row("Connection", palette)
        .id("settings-connection")
        .child(
            div()
                .flex_1()
                .min_w_0()
                .truncate()
                .text_size(px(12.))
                .text_color(match connection {
                    Connection::Failed(_) => palette.red,
                    _ => palette.text2,
                })
                .child(note),
        )
        .child(text_button(
            "engine-reconnect",
            "Reconnect",
            ButtonTone::Accent,
            !matches!(connection, Connection::Connecting),
            palette,
            move |_, _, cx| engine_source::reconnect(&workspace, cx),
        ))
        .help("Connect to the engine again.")
}
