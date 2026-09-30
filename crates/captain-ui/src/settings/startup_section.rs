//! The Startup section: open Captain at login, and the menu bar icon. Start in the
//! background lives in the settings file. See features 0015 and 0037.

use captain_core::settings::Settings;
use gpui_kit::component::switch::Switch;
use gpui_kit::*;

use super::page_section::{row, section, under_note};
use super::{SettingsView, store, system};
use crate::help::HelpExt;
use crate::theme::Palette;

/// Where the icon lives on this platform.
const ICON_PLACE: &str = if cfg!(target_os = "windows") {
    "notification area"
} else {
    "menu bar"
};

/// The section, or `None` when the app installed no system integration.
pub fn render(
    view: &SettingsView,
    settings: &Settings,
    palette: &Palette,
    cx: &mut Context<SettingsView>,
) -> Option<Div> {
    let system = system::system(cx)?;
    let (enabled, read_error) = match system.login_item() {
        Ok(enabled) => (enabled, None),
        Err(error) => (
            false,
            Some(format!("Captain cannot read the login item. {error}")),
        ),
    };
    let this = cx.weak_entity();
    let login =
        row("Startup", palette)
            .id("settings-login")
            .child(div().flex_1().child("Open Captain at login"))
            .child(Switch::new("behavior-login").checked(enabled).on_click(
                move |checked, _, cx| {
                    let checked = *checked;
                    let result = system::system(cx).map(|system| system.set_login_item(checked));
                    this.update(cx, |view, cx| {
                        view.login_error = match result {
                            Some(Err(error)) => {
                                tracing::warn!(%error, "cannot change the login item");
                                Some(
                                    format!("Captain cannot change the login item. {error}").into(),
                                )
                            }
                            _ => None,
                        };
                        cx.notify();
                    })
                    .ok();
                },
            ))
            .help("Open Captain when you log in to this computer.");
    let error = view.login_error.clone().or(read_error.map(Into::into));
    let icon = system.has_menu_bar_icon().then(|| {
        row("", palette)
            .id("settings-menu-bar-icon")
            .child(div().flex_1().child(format!("Show Captain in the {ICON_PLACE}")))
            .child(
                Switch::new("behavior-menu-bar-icon")
                    .checked(settings.show_menu_bar_icon)
                    .on_click(|checked, _, cx| {
                        let checked = *checked;
                        store::update(cx, |settings| settings.show_menu_bar_icon = checked);
                    }),
            )
            .help(format!(
                "Show Captain's icon in the {ICON_PLACE}. Without it, closing the window quits Captain."
            ))
    });
    Some(
        section(palette)
            .child(login)
            .children(error.map(|error| under_note(error, palette).text_color(palette.red)))
            .children(icon),
    )
}
