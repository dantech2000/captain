use captain_core::format::percent_label;
use gpui_kit::*;

use super::ImagesState;
use crate::theme::Palette;
use crate::widgets::progress_bar;

/// One pull or push as the status row shows it.
struct Transfer<'a> {
    reference: &'a str,
    status: &'a str,
    layers: (usize, usize),
    fraction: f32,
    finished: bool,
}

/// The running or last pull and push: reference, status line, layer count, and a
/// progress bar. A failed transfer shows its error in red too.
pub fn render(state: &ImagesState, palette: &Palette) -> Vec<Div> {
    let pull = state.pull().map(|pull| Transfer {
        reference: pull.reference(),
        status: pull.status(),
        layers: pull.layer_counts(),
        fraction: pull.fraction(),
        finished: pull.is_finished(),
    });
    let push = state.push().map(|push| Transfer {
        reference: push.reference(),
        status: push.status(),
        layers: push.layer_counts(),
        fraction: push.fraction(),
        finished: push.is_finished(),
    });
    [
        row(pull, state.pull_error(), palette),
        row(push, state.push_error(), palette),
    ]
    .into_iter()
    .flatten()
    .collect()
}

/// The row for one transfer, or `None` when there is nothing to show.
fn row(transfer: Option<Transfer>, error: Option<&str>, palette: &Palette) -> Option<Div> {
    if transfer.is_none() && error.is_none() {
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

    let Some(transfer) = transfer else {
        return Some(row.child(message(error.unwrap_or_default(), palette.red)));
    };

    let (done, total) = transfer.layers;
    let layers = match total {
        0 => String::new(),
        1 => format!("{done} of 1 layer"),
        n => format!("{done} of {n} layers"),
    };
    let color = match (error, transfer.finished) {
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
                .child(transfer.reference.to_string()),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .truncate()
                .text_color(palette.text2)
                .child(transfer.status.to_string()),
        )
        .child(div().text_color(palette.text3).child(layers))
        .child(
            div()
                .w(px(40.))
                .text_right()
                .text_color(palette.text2)
                .child(percent_label(f64::from(transfer.fraction) * 100.0)),
        );

    Some(
        row.child(title)
            .child(progress_bar(transfer.fraction, color, palette))
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
