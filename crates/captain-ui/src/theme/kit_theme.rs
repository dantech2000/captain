//! Gives gpui-kit's own components (inputs, switches, selects, notifications) the
//! Captain theme. See https://gpui-kit.com/component/theme.

use std::rc::Rc;

use captain_core::settings::ThemeFamily;
use gpui_kit::component::highlighter::HighlightThemeStyle;
use gpui_kit::component::{Theme, ThemeConfig, ThemeConfigColors, ThemeMode};
use gpui_kit::*;
use serde_json::{Value, json};

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
    // gpui-kit stacks toasts like Sonner: older ones peek out below the newest at
    // their own height, so a long error shows its tail under a short toast. One toast
    // at a time; an older one shows again if it has time left.
    theme.notification.max_items = MAX_TOASTS;
}

/// The toasts a window shows at once.
const MAX_TOASTS: usize = 1;

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
        highlight: Some(highlight(&t)),
        ..ThemeConfig::default()
    }
}

/// The code editor theme. Without it the editor keeps gpui-kit's default light
/// highlight theme in every mode, white background included.
fn highlight(t: &Tokens) -> HighlightThemeStyle {
    let s = t.syntax();
    let hex = |color: u32| format!("#{color:06x}");
    let color = |color: u32| json!({ "color": hex(color) });
    let punctuation = color(s.punctuation);
    let mut syntax = serde_json::Map::new();
    for (names, style) in [
        (&["property"][..], color(s.key)),
        (&["string", "text.literal"], color(s.string)),
        (
            &["string.escape", "number", "boolean", "constant"],
            color(s.number),
        ),
        (&["keyword", "type", "label", "attribute"], color(s.keyword)),
        (
            &["comment"],
            json!({ "color": hex(s.comment), "font_style": "italic" }),
        ),
        (&["punctuation", "operator"], punctuation),
    ] {
        for name in names {
            syntax.insert((*name).into(), style.clone());
        }
    }
    let style = json!({
        "editor.background": hex(s.background),
        "editor.gutter.background": hex(s.background),
        "editor.foreground": hex(s.text),
        "editor.active_line.background": format!("#{:06x}66", t.border_strong),
        "editor.line_number": hex(s.line_number),
        "editor.active_line_number": hex(s.active_line_number),
        "editor.invisible": format!("#{:06x}99", t.text3),
        "error": hex(t.failing),
        "warning": hex(t.warning),
        "info": hex(t.info),
        "syntax": Value::Object(syntax),
    });
    serde_json::from_value(style).expect("the editor theme matches gpui-kit's schema")
}

#[cfg(test)]
mod tests;
