use gpui_kit::*;

use crate::theme::Palette;

/// The build output in a monospace font, in a box of fixed height. The dialog
/// scrolls it to the end as lines arrive.
pub fn log_view(
    lines: &[SharedString],
    scroll: &UniformListScrollHandle,
    palette: &Palette,
) -> Div {
    let lines = lines.to_vec();
    let color = palette.text2;
    div()
        .h(px(220.))
        .flex()
        .flex_col()
        .px(px(10.))
        .py(px(8.))
        .rounded(px(8.))
        .bg(palette.field)
        .border_1()
        .border_color(palette.sep)
        .font_family(palette.mono())
        .text_size(px(11.))
        .child(
            uniform_list("build-log", lines.len(), move |range, _, _| {
                range
                    .map(|ix| {
                        div()
                            .text_color(color)
                            .whitespace_nowrap()
                            .child(lines[ix].clone())
                    })
                    .collect()
            })
            .track_scroll(scroll)
            .flex_1(),
        )
}
