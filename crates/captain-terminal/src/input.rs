/// A key the user pressed, in terms the encoder understands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyInput {
    pub key: Key,
    pub mods: Modifiers,
    /// The text the key types with its modifiers, for example `A` for shift-a. `None`
    /// means the encoder uses the character of [`Key::Char`].
    pub text: Option<String>,
}

impl KeyInput {
    pub fn new(key: Key) -> Self {
        Self {
            key,
            mods: Modifiers::default(),
            text: None,
        }
    }

    pub fn with_mods(mut self, mods: Modifiers) -> Self {
        self.mods = mods;
        self
    }

    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }
}

/// A key with no modifiers applied. Letters are lowercase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Char(char),
    Enter,
    Tab,
    Backspace,
    Escape,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    Insert,
    Delete,
    /// A function key, 1 to 12.
    F(u8),
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Modifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}

impl Modifiers {
    pub const CTRL: Self = Self {
        ctrl: true,
        alt: false,
        shift: false,
    };
    pub const ALT: Self = Self {
        ctrl: false,
        alt: true,
        shift: false,
    };
    pub const SHIFT: Self = Self {
        ctrl: false,
        alt: false,
        shift: true,
    };

    pub fn any(self) -> bool {
        self.ctrl || self.alt || self.shift
    }

    /// The xterm modifier parameter: 1 plus shift 1, alt 2, and ctrl 4.
    pub fn xterm_param(self) -> u8 {
        1 + u8::from(self.shift) + 2 * u8::from(self.alt) + 4 * u8::from(self.ctrl)
    }
}

/// The terminal modes that change what keys, pastes, and the scroll wheel send.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct InputModes {
    /// Arrows, Home, and End send `ESC O` sequences (DECCKM).
    pub app_cursor: bool,
    /// Pastes are wrapped in `ESC [200~` and `ESC [201~`.
    pub bracketed_paste: bool,
    /// Enter sends CR LF (LNM).
    pub newline: bool,
    /// A full-screen program such as `less` or `vim` uses the alternate screen.
    pub alt_screen: bool,
    /// On the alternate screen, the scroll wheel sends arrow keys.
    pub alternate_scroll: bool,
}
