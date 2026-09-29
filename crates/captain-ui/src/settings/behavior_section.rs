//! The Behavior card: start at login, start in the background, and the menu bar icon.
//! See feature 0015.

use captain_core::settings::Settings;
use gpui_kit::component::Disableable;
use gpui_kit::component::switch::Switch;
use gpui_kit::*;

use super::{SettingsView, store, system};
use crate::theme::Palette;
use crate::widgets::{settings_card, settings_row};

/// What Windows calls the menu bar icon.
const ICON_NAME: &str = if cfg!(target_os = "windows") {
    "notification area icon"
} else {
    "menu bar icon"
};

/// The card, or `None` when the app installed no system integration.
pub fn render(
    view: &SettingsView,
    settings: &Settings,
    palette: &Palette,
    cx: &mut Context<SettingsView>,
) -> Option<Div> {
    let system = system::system(cx)?;
    let mut rows = vec![login_row(view, system.login_item(), palette, cx)];
    if system.has_menu_bar_icon() {
        rows.push(background_row(settings, palette));
        rows.push(icon_row(settings, palette));
    }
    Some(settings_card("Behavior", rows, palette))
}

/// The switch shows the login item that the system has now, so a change made outside
/// Captain shows up the next time the page draws.
fn login_row(
    view: &SettingsView,
    state: Result<bool, String>,
    palette: &Palette,
    cx: &mut Context<SettingsView>,
) -> AnyElement {
    let (enabled, read_error) = match state {
        Ok(enabled) => (enabled, None),
        Err(error) => (
            false,
            Some(format!("Captain cannot read the login item. {error}")),
        ),
    };
    let note = view
        .login_error
        .clone()
        .or(read_error.map(Into::into))
        .unwrap_or_else(|| "Captain starts when you log in.".into());
    let this = cx.weak_entity();
    settings_row(
        "Start at login",
        Some(note),
        Switch::new("behavior-login")
            .checked(enabled)
            .on_click(move |checked, _, cx| {
                let checked = *checked;
                let result = system::system(cx).map(|system| system.set_login_item(checked));
                this.update(cx, |view, cx| {
                    view.login_error = match result {
                        Some(Err(error)) => {
                            tracing::warn!(%error, "cannot change the login item");
                            Some(format!("Captain cannot change the login item. {error}").into())
                        }
                        _ => None,
                    };
                    cx.notify();
                })
                .ok();
            }),
        palette,
    )
    .into_any_element()
}

fn background_row(settings: &Settings, palette: &Palette) -> AnyElement {
    let note = if settings.show_menu_bar_icon {
        format!("Captain opens only its {ICON_NAME}, also at login.")
    } else {
        format!("Needs the {ICON_NAME}. Without it, the window always opens.")
    };
    settings_row(
        "Start in the background",
        Some(note.into()),
        Switch::new("behavior-background")
            .checked(settings.start_in_background)
            .disabled(!settings.show_menu_bar_icon)
            .on_click(|checked, _, cx| {
                let checked = *checked;
                store::update(cx, |settings| settings.start_in_background = checked);
            }),
        palette,
    )
    .into_any_element()
}

fn icon_row(settings: &Settings, palette: &Palette) -> AnyElement {
    settings_row(
        format!("Show the {ICON_NAME}"),
        Some("When it is off, closing the window quits Captain.".into()),
        Switch::new("behavior-menu-bar-icon")
            .checked(settings.show_menu_bar_icon)
            .on_click(|checked, _, cx| {
                let checked = *checked;
                store::update(cx, |settings| settings.show_menu_bar_icon = checked);
            }),
        palette,
    )
    .into_any_element()
}
