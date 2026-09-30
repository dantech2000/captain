use gpui_kit::assets::IconName;
use gpui_kit::component::Icon;
use gpui_kit::*;

use super::CaptainIcon;

/// The body fill's opacity, and the second fill's.
const FILL: f32 = 0.22;
const SECOND_FILL: f32 = 0.10;
/// At this size and below the fills blur, so only the line drawing is drawn.
const LINE_ONLY: f32 = 12.;

/// `icon` in duotone: the fills in a soft tint of `color` under the line drawing.
/// GPUI draws an SVG as a one-color mask, so each tone is its own layer.
pub fn cap_icon(icon: CaptainIcon, size: Pixels, color: Hsla) -> Div {
    let layer = |path: &'static str, color: Hsla| {
        svg()
            .path(path)
            .absolute()
            .top_0()
            .left_0()
            .size(size)
            .text_color(color)
    };
    let base = div().relative().flex_shrink_0().size(size);
    if size <= px(LINE_ONLY) {
        return base.child(layer(icon.line_path(), color));
    }
    base.children(
        icon.second_fill_path()
            .map(|path| layer(path, color.opacity(SECOND_FILL))),
    )
    .child(layer(icon.fill_path(), color.opacity(FILL)))
    .child(layer(icon.line_path(), color))
}

/// A Captain glyph for a resource, or a Lucide glyph for anything else.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Glyph {
    Captain(CaptainIcon),
    Lucide(IconName),
}

impl From<CaptainIcon> for Glyph {
    fn from(icon: CaptainIcon) -> Self {
        Glyph::Captain(icon)
    }
}

impl From<IconName> for Glyph {
    fn from(icon: IconName) -> Self {
        Glyph::Lucide(icon)
    }
}

/// `glyph` at `size` in `color`.
pub fn glyph(glyph: impl Into<Glyph>, size: Pixels, color: Hsla) -> AnyElement {
    match glyph.into() {
        Glyph::Captain(icon) => cap_icon(icon, size, color).into_any_element(),
        Glyph::Lucide(icon) => Icon::new(icon)
            .size(size)
            .text_color(color)
            .into_any_element(),
    }
}
