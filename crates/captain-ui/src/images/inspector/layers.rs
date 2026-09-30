use captain_core::format::bytes_label;
use captain_core::model::{ImageLayer, largest_layer_size};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use crate::help::HelpExt;
use crate::theme::Palette;
use crate::widgets::{section_note, titled_section};

/// The image history, newest first. Each step shows its command in one line, with the
/// whole command in a tooltip, and a bar scaled to the largest layer.
pub fn render(layers: &[ImageLayer], palette: &Palette) -> Div {
    let largest = largest_layer_size(layers);
    let note = match layers.len() {
        0 => None,
        1 => Some("1 step".to_string()),
        n => Some(format!("{n} steps")),
    };
    let body = if layers.is_empty() {
        section_note("No history. The image was imported, not built.", palette)
    } else {
        div()
            .flex()
            .flex_col()
            .rounded(px(9.))
            .border_1()
            .border_color(palette.sep)
            .overflow_hidden()
            .children(
                layers
                    .iter()
                    .enumerate()
                    .map(|(ix, layer)| row(ix, layer, largest, palette)),
            )
    };
    titled_section("Layers", note, body, palette)
}

fn row(ix: usize, layer: &ImageLayer, largest: u64, palette: &Palette) -> Stateful<Div> {
    let command = layer.command();
    let full = SharedString::from(if command.is_empty() {
        "(no command)".to_string()
    } else {
        command
    });
    let size = bytes_label(layer.size);

    div()
        .id(("image-layer", ix))
        .flex()
        .flex_col()
        .gap(px(5.))
        .px(px(10.))
        .py(px(7.))
        .when(ix > 0, |row| row.border_t_1().border_color(palette.sep))
        .help(full.clone())
        .child(
            div()
                .flex()
                .gap(px(10.))
                .text_size(px(11.))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .truncate()
                        .font_family(palette.mono())
                        .text_color(if layer.size == 0 {
                            palette.text2
                        } else {
                            palette.text
                        })
                        .child(full),
                )
                .child(div().flex_shrink_0().text_color(palette.text2).child(size)),
        )
        .child(
            div()
                .h(px(4.))
                .w_full()
                .rounded(px(2.))
                .bg(palette.track)
                .child(
                    div()
                        .h_full()
                        .w(relative(layer.size_fraction(largest)))
                        .rounded(px(2.))
                        .bg(palette.accent),
                ),
        )
}
