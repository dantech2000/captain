use captain_core::settings::Accent;
use gpui_kit::*;

/// The color of an accent preset. Light mode uses darker shades, because the accent
/// also colors text on white.
pub fn accent_color(accent: Accent, dark: bool) -> Hsla {
    let (dark_hex, light_hex) = match accent {
        Accent::Blue => (0x0a84ff, 0x007aff),
        Accent::Purple => (0xbf5af2, 0x9b44c9),
        Accent::Orange => (0xff9f0a, 0xc45a00),
        Accent::Teal => (0x40c8e0, 0x0a7f96),
        Accent::Graphite => (0x98989d, 0x6e6e73),
    };
    rgb(if dark { dark_hex } else { light_hex }).into()
}

#[cfg(test)]
mod tests;
