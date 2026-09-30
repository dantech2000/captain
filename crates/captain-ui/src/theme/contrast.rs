//! WCAG contrast, from https://www.w3.org/TR/WCAG21/#dfn-contrast-ratio.

/// The WCAG 2 contrast ratio of two opaque colors, from 1 to 21.
pub fn contrast(a: u32, b: u32) -> f32 {
    let (a, b) = (luminance(a), luminance(b));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

/// WCAG relative luminance of `0xRRGGBB`.
fn luminance(hex: u32) -> f32 {
    let channel = |shift: u32| {
        let c = ((hex >> shift) & 0xff) as f32 / 255.;
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * channel(16) + 0.7152 * channel(8) + 0.0722 * channel(0)
}
