use captain_core::format::percent_label;
use gpui_kit::*;

use super::ImagesState;
use crate::theme::Palette;
use crate::widgets::progress_bar;

/// The running or last pull: reference, status line, layer count, and a progress bar.
/// A failed pull shows its error in red instead. `None` when there is nothing to show.
pub fn render(state: &ImagesState, palette: &Palette) -> Option<Div> {
    let pull = state.pull();
    let error = state.pull_error();
    if pull.is_none() && error.is_none() {
        return None;
    }

    let row = div()
        .flex_shrink_0()
        .mx(px(24.))
        .mb(px(12.))
        .px(px(12.))
        .py(px(10.))
        .flex()
        .flex_col()
        .gap(px(6.))
        .rounded(px(10.))
        .bg(palette.card)
        .border_1()
        .border_color(palette.sep)
        .text_size(px(12.));

    let Some(pull) = pull else {
        return Some(row.child(message(error.unwrap_or_default(), palette.red)));
    };

    let (done, total) = pull.layer_counts();
    let layers = match total {
        0 => String::new(),
        1 => format!("{done} of 1 layer"),
        n => format!("{done} of {n} layers"),
    };
    let color = match (error, pull.is_finished()) {
        (Some(_), _) => palette.red,
        (None, true) => palette.green,
        (None, false) => palette.accent,
    };

    let title = div()
        .flex()
        .items_center()
        .gap(px(8.))
        .child(
            div()
                .font_family(palette.mono())
                .font_weight(FontWeight::SEMIBOLD)
                .child(pull.reference().to_string()),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .truncate()
                .text_color(palette.text2)
                .child(pull.status().to_string()),
        )
        .child(div().text_color(palette.text3).child(layers))
        .child(
            div()
                .w(px(40.))
                .text_right()
                .text_color(palette.text2)
                .child(percent_label(f64::from(pull.fraction()) * 100.0)),
        );

    Some(
        row.child(title)
            .child(progress_bar(pull.fraction(), color, palette))
            .children(error.map(|error| message(error, palette.red))),
    )
}

/// A one-line message in `color`, for errors and results.
pub fn message(text: &str, color: Hsla) -> Div {
    div()
        .text_size(px(12.))
        .text_color(color)
        .child(text.to_string())
}
