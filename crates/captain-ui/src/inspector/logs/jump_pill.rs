use gpui_kit::assets::IconName;
use gpui_kit::component::Icon;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use crate::theme::Palette;

/// The floating "Jump to latest" button that shows while following is paused.
/// `unseen` counts the new lines that pass the filters.
pub fn jump_pill(
    unseen: usize,
    palette: &Palette,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let hover = palette.accent.opacity(0.85);
    div()
        .id("logs-jump-latest")
        .h(px(26.))
        .px(px(12.))
        .flex()
        .items_center()
        .gap(px(5.))
        .rounded(px(13.))
        .bg(palette.accent)
        .text_color(palette.on_accent)
        .text_size(px(11.))
        .font_weight(FontWeight::SEMIBOLD)
        .shadow(vec![BoxShadow {
            color: hsla(0., 0., 0., if palette.dark { 0.45 } else { 0.18 }),
            offset: point(px(0.), px(2.)),
            blur_radius: px(8.),
            spread_radius: px(0.),
            inset: false,
        }])
        .cursor_pointer()
        .hover(move |style| style.bg(hover))
        .on_click(on_click)
        .child("Jump to latest")
        .child(Icon::new(IconName::ArrowDown).size(px(12.)))
        .when(unseen > 0, |pill| pill.child(format!("({unseen} new)")))
}
