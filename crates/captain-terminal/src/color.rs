/// A 24-bit color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

/// The color of a cell's text or background. The view picks the actual colors for the
/// defaults and the ANSI colors, so they can follow the app theme.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Color {
    /// The default text color.
    #[default]
    Foreground,
    /// The default background color.
    Background,
    /// An entry of the 256-color palette. 0 to 15 are the ANSI colors, 16 to 231 the
    /// 6×6×6 cube, and 232 to 255 the gray ramp.
    Indexed(u8),
    /// A true color set by the program, or a palette entry the program redefined.
    Rgb(Rgb),
}

impl Color {
    /// The xterm value of a palette entry above 15. The ANSI colors (0 to 15) have no
    /// fixed value, so they give `None`.
    pub fn xterm_rgb(index: u8) -> Option<Rgb> {
        match index {
            0..=15 => None,
            16..=231 => {
                let i = index - 16;
                let level = |v: u8| if v == 0 { 0 } else { 55 + v * 40 };
                Some(Rgb::new(level(i / 36), level((i / 6) % 6), level(i % 6)))
            }
            _ => {
                let gray = 8 + (index - 232) * 10;
                Some(Rgb::new(gray, gray, gray))
            }
        }
    }
}

#[cfg(test)]
mod tests;
