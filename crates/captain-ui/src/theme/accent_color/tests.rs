use captain_core::settings::Accent;

use super::accent_color;

#[test]
fn every_accent_differs_in_each_mode() {
    for dark in [false, true] {
        let colors: Vec<_> = Accent::ALL
            .into_iter()
            .map(|accent| accent_color(accent, dark))
            .collect();
        for (ix, color) in colors.iter().enumerate() {
            assert!(!colors[ix + 1..].contains(color), "dark: {dark}, {ix}");
        }
    }
}

#[test]
fn light_mode_uses_darker_shades() {
    for accent in Accent::ALL {
        let light = accent_color(accent, false);
        let dark = accent_color(accent, true);
        assert!(light.l < dark.l, "{accent:?}");
    }
}
