use gpui_kit::*;

use super::CommandPalette;
use crate::help::HelpExt;
use crate::theme::Palette;

/// The first row of the empty palette: "Try" and commands built from live names.
/// A click puts the command in the search field.
pub fn render(
    examples: Vec<String>,
    palette: &Palette,
    cx: &mut Context<CommandPalette>,
) -> impl IntoElement {
    let hover = palette.nav_selected;
    let chips = examples.into_iter().enumerate().map(|(ix, line)| {
        let help = format!("Fill in \u{201c}{line}\u{201d}. Press Return to run it.");
        let fill = line.clone();
        div()
            .id(("palette-try", ix))
            .h(px(22.))
            .px(px(8.))
            .flex()
            .items_center()
            .rounded(px(6.))
            .border_1()
            .border_color(palette.sep)
            .bg(palette.field)
            .font_family(palette.mono())
            .text_size(px(11.))
            .text_color(palette.text2)
            .cursor_pointer()
            .hover(move |style| style.bg(hover))
            .on_click(cx.listener(move |this, _, window, cx| this.fill(&fill, window, cx)))
            .help(help)
            .child(line)
    });
    div()
        .flex_shrink_0()
        .flex()
        .flex_wrap()
        .items_center()
        .gap(px(6.))
        .px(px(18.))
        .py(px(10.))
        .border_b_1()
        .border_color(palette.sep)
        .child(
            div()
                .mr(px(4.))
                .text_size(px(11.))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(palette.text3)
                .child("Try"),
        )
        .children(chips)
}
