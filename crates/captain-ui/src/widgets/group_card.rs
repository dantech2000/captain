use gpui_kit::assets::IconName;
use gpui_kit::component::Icon;
use gpui_kit::*;

use crate::icons::{CaptainIcon, cap_icon};
use crate::theme::Palette;

/// A rounded card for one Compose project, or for the items with no project, with a
/// header that names it and a summary on the right.
pub fn group_card(
    project: Option<&str>,
    summary: impl Into<SharedString>,
    rows: impl IntoIterator<Item = AnyElement>,
    palette: &Palette,
) -> Div {
    let (name, color, icon) = match project {
        Some(name) => (
            name.to_string(),
            palette.project_color(name),
            CaptainIcon::Stack,
        ),
        None => (
            "Not in a project".to_string(),
            palette.gray,
            CaptainIcon::Container,
        ),
    };
    let header = div()
        .h(px(38.))
        .px(px(10.))
        .flex()
        .items_center()
        .gap(px(10.))
        .child(
            Icon::new(IconName::ChevronDown)
                .size(px(12.))
                .text_color(palette.text3),
        )
        .child(cap_icon(icon, px(20.), color))
        .child(div().font_weight(FontWeight::SEMIBOLD).child(name))
        .child(div().flex_1())
        .child(
            div()
                .text_size(px(11.))
                .text_color(palette.text2)
                .child(summary.into()),
        );

    div()
        .flex()
        .flex_col()
        .p(px(4.))
        .rounded(px(12.))
        .bg(palette.group)
        .border_1()
        .border_color(palette.sep)
        .child(header)
        .children(rows)
}
