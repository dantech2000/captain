//! Gives gpui-kit's own components (inputs, switches, selects, notifications) the
//! Captain theme. See https://gpui-kit.com/component/theme.

use std::rc::Rc;

use captain_core::settings::ThemeFamily;
use gpui_kit::component::{Theme, ThemeConfig, ThemeConfigColors, ThemeMode};
use gpui_kit::*;

use super::Tokens;

/// Registers the light and dark gpui-kit themes for `family`. The next
/// `Theme::change` loads the one for its mode; [`crate::settings::apply_appearance`]
/// calls it right after.
pub fn install_kit_themes(family: ThemeFamily, cx: &mut App) {
    if !cx.has_global::<Theme>() {
        return;
    }
    let theme = Theme::global_mut(cx);
    theme.light_theme = Rc::new(config(family, false));
    theme.dark_theme = Rc::new(config(family, true));
}

fn config(family: ThemeFamily, dark: bool) -> ThemeConfig {
    let t = Tokens::of(family, dark);
    let hex = |color: u32| Some(format!("#{color:06x}").into());
    let alpha = |color: u32, alpha: u8| Some(format!("#{color:06x}{alpha:02x}").into());
    let mut colors = ThemeConfigColors::default();
    colors.background = hex(t.window);
    colors.foreground = hex(t.text);
    colors.border = hex(t.border);
    colors.input = hex(t.border_strong);
    colors.ring = hex(t.action);
    colors.caret = hex(t.action);
    colors.selection = alpha(t.action, 0x4c);
    colors.primary = hex(t.action);
    colors.primary_foreground = hex(t.on_action);
    colors.secondary = hex(t.button);
    colors.secondary_foreground = hex(t.text);
    colors.button = hex(t.button);
    colors.button_foreground = hex(t.text);
    colors.muted = hex(t.field);
    colors.muted_foreground = hex(t.text3);
    colors.accent = hex(t.border);
    colors.accent_foreground = hex(t.text);
    colors.popover = hex(t.card);
    colors.popover_foreground = hex(t.text);
    colors.list = hex(t.card);
    colors.list_active = alpha(t.action, 0x33);
    colors.list_active_border = hex(t.action);
    colors.link = hex(t.link);
    colors.success = hex(t.running);
    colors.success_foreground = hex(t.on(t.running));
    colors.warning = hex(t.warning);
    colors.warning_foreground = hex(t.on(t.warning));
    colors.danger = hex(t.failing);
    colors.danger_foreground = hex(t.on(t.failing));
    colors.info = hex(t.info);
    colors.info_foreground = hex(t.on(t.info));
    colors.switch = hex(t.border_strong);
    colors.switch_thumb = hex(0xFFFFFF);
    colors.slider_bar = hex(t.action);
    colors.slider_thumb = hex(t.on_action);
    colors.progress_bar = hex(t.action);
    colors.scrollbar_thumb = hex(t.border_strong);
    colors.scrollbar_thumb_hover = hex(t.text3);
    colors.sidebar = hex(t.sidebar);
    colors.sidebar_foreground = hex(t.text);
    colors.sidebar_border = hex(t.border);
    colors.sidebar_primary = hex(t.action);
    colors.sidebar_primary_foreground = hex(t.on_action);
    colors.window_border = hex(t.border);
    ThemeConfig {
        name: format!(
            "Captain {} {}",
            family.label(),
            if dark { "Dark" } else { "Light" }
        )
        .into(),
        mode: if dark {
            ThemeMode::Dark
        } else {
            ThemeMode::Light
        },
        colors,
        ..ThemeConfig::default()
    }
}
