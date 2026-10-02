use gpui_kit::component::empty::{Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle};
use gpui_kit::*;

use crate::icons::{CaptainIcon, cap_icon};
use crate::theme::Palette;

/// A centered icon, title, and hint for an empty list, on gpui-kit's Empty with
/// Captain's sizes and colors. See https://gpui-kit.com/component/empty.
pub fn empty_note(
    icon: CaptainIcon,
    title: impl Into<SharedString>,
    hint: impl Into<SharedString>,
    palette: &Palette,
) -> Div {
    let header = EmptyHeader::new()
        .media(
            EmptyMedia::new()
                .mb_0()
                .child(cap_icon(icon, px(32.), palette.text3)),
        )
        .title(
            EmptyTitle::new()
                .text_size(px(13.))
                .text_color(palette.text)
                .font_weight(FontWeight::SEMIBOLD)
                .child(title.into()),
        )
        .description(
            EmptyDescription::new()
                .text_size(px(12.))
                .line_height(phi())
                .text_color(palette.text2)
                .child(hint.into()),
        );
    div().w_full().child(
        Empty::new()
            .flex_none()
            .justify_start()
            .p_0()
            .pt(px(80.))
            .header(header),
    )
}
