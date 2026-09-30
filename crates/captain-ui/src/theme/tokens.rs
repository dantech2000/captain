//! The theme token tables from docs/features/0027-v3-interface.md, as hex colors.

use captain_core::settings::ThemeFamily;

use super::contrast;

/// The colors of one theme in one mode. [`super::Palette`] and the gpui-kit theme
/// are both built from these.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tokens {
    pub window: u32,
    pub sidebar: u32,
    pub card: u32,
    pub field: u32,
    pub button: u32,
    pub border: u32,
    pub border_strong: u32,
    pub text: u32,
    pub text2: u32,
    pub text3: u32,
    /// Buttons, links, focus, and selection. Never a state.
    pub action: u32,
    pub on_action: u32,
    pub link: u32,
    pub running: u32,
    pub warning: u32,
    pub warning_text: u32,
    pub failing: u32,
    pub info: u32,
    pub log_panel: u32,
    /// Only for telling projects apart.
    pub labels: [u32; 5],
}

/// Harbor's neutrals, which Dusk shares.
const HARBOR_DARK: Tokens = Tokens {
    window: 0x1b2632,
    sidebar: 0x16202a,
    card: 0x202d3b,
    field: 0x223040,
    button: 0x2c3b4d,
    border: 0x2e3d4f,
    border_strong: 0x3c4e64,
    text: 0xeee9df,
    text2: 0xc9c1b1,
    text3: 0x9a9486,
    action: 0xffb162,
    on_action: 0x1b2632,
    link: 0xffb162,
    running: 0x8ccb9b,
    warning: 0xf2d06b,
    warning_text: 0xf4d987,
    failing: 0xe8735a,
    info: 0x8fb4d9,
    log_panel: 0x141d27,
    labels: [0xc9785c, 0x8fb4d9, 0xd4a5c9, 0xe6b980, 0xc9c1b1],
};

const HARBOR_LIGHT: Tokens = Tokens {
    window: 0xf4f0e8,
    sidebar: 0xeae4d8,
    card: 0xfbf8f2,
    field: 0xeee9df,
    button: 0xfbf8f2,
    border: 0xdad2c3,
    border_strong: 0xc9c1b1,
    text: 0x1b2632,
    text2: 0x3e4b5b,
    text3: 0x56606b,
    action: 0xffb162,
    on_action: 0x1b2632,
    link: 0xa35139,
    running: 0x2f7a4d,
    warning: 0xc8960f,
    warning_text: 0x7a5a00,
    failing: 0xb8452c,
    info: 0x2f5e8c,
    log_panel: 0xfbf8f2,
    labels: [0xa35139, 0x2f5e8c, 0x8a4f7d, 0xb06a1e, 0x3f7a55],
};

const DUSK_DARK: Tokens = Tokens {
    action: 0x5f5ff0,
    on_action: 0xffffff,
    link: 0xb8baff,
    running: 0xb9f0d7,
    warning: 0xffb162,
    warning_text: 0xffc48a,
    labels: [0xc9785c, 0x8fb4d9, 0xd4a5c9, 0xe6b980, 0xb9f0d7],
    ..HARBOR_DARK
};

const DUSK_LIGHT: Tokens = Tokens {
    action: 0x5c5cf7,
    on_action: 0xffffff,
    link: 0x4747de,
    running: 0x2f7a4d,
    warning: 0xd98a0b,
    warning_text: 0x8f5a00,
    ..HARBOR_LIGHT
};

const PERIWINKLE_DARK: Tokens = Tokens {
    window: 0x0c0c16,
    sidebar: 0x11111d,
    card: 0x141424,
    field: 0x171729,
    button: 0x1c1c31,
    border: 0x25253c,
    border_strong: 0x33334f,
    text: 0xedeeff,
    text2: 0xa9abd6,
    text3: 0x7e80ae,
    // The canvas has #6666FF, which gives white text only 4.28:1.
    action: 0x6060ff,
    on_action: 0xffffff,
    link: 0xb8baff,
    running: 0xb9f0d7,
    warning: 0xffc46b,
    warning_text: 0xffd08a,
    failing: 0xff7a85,
    info: 0xc9e8ff,
    log_panel: 0x08080f,
    labels: [0x9d8cff, 0x7fddc0, 0xff8fc7, 0xffb36b, 0xc9e8ff],
};

const PERIWINKLE_LIGHT: Tokens = Tokens {
    window: 0xffffff,
    sidebar: 0xf5f5ff,
    card: 0xffffff,
    field: 0xf1f1fb,
    button: 0xefeffa,
    border: 0xe1e1f2,
    border_strong: 0xcfcfe6,
    text: 0x0c0c16,
    text2: 0x474868,
    text3: 0x5d5f85,
    action: 0x5c5cf7,
    on_action: 0xffffff,
    link: 0x4747de,
    running: 0x1e8a5c,
    warning: 0xd98a0b,
    warning_text: 0x8f5a00,
    failing: 0xd23a4b,
    info: 0x2c7fb8,
    log_panel: 0xf8f8fe,
    labels: [0x7b61ff, 0x1f9e83, 0xc23f8a, 0xc26a12, 0x2c7fb8],
};

impl Tokens {
    pub fn of(family: ThemeFamily, dark: bool) -> Self {
        match (family, dark) {
            (ThemeFamily::Dusk, true) => DUSK_DARK,
            (ThemeFamily::Dusk, false) => DUSK_LIGHT,
            (ThemeFamily::Periwinkle, true) => PERIWINKLE_DARK,
            (ThemeFamily::Periwinkle, false) => PERIWINKLE_LIGHT,
            (ThemeFamily::Harbor, true) => HARBOR_DARK,
            (ThemeFamily::Harbor, false) => HARBOR_LIGHT,
        }
    }

    /// Whichever of the text and window colors reads better on `fill`, for a
    /// word on a solid state color.
    pub fn on(&self, fill: u32) -> u32 {
        if contrast(self.text, fill) >= contrast(self.window, fill) {
            self.text
        } else {
            self.window
        }
    }
}

#[cfg(test)]
mod tests;
