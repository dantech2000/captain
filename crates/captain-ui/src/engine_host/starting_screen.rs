//! Shown while Captain Engine starts or stops: a moving bar and the last progress
//! lines.

use captain_core::HostStatus;
use gpui_kit::component::progress::Progress;
use gpui_kit::*;

use super::HostModel;
use super::host_screen::{frame, note, page, title};
use crate::theme::Palette;

/// How many progress lines to show.
const LINES: usize = 8;

pub fn render(host: &HostModel, palette: &Palette) -> Stateful<Div> {
    let stopping = *host.status() == HostStatus::Stopping;
    let (heading, hint) = if stopping {
        ("Stopping Captain Engine", "This takes a few seconds.")
    } else {
        (
            "Starting Captain Engine",
            "The first start downloads about 600 MB and installs Docker. Later starts \
             take 10 to 20 seconds.",
        )
    };
    let column = frame(palette)
        .child(title(heading))
        .child(note(hint, palette))
        .child(
            div().w(px(360.)).pt(px(4.)).child(
                Progress::new("host-progress")
                    .loading(true)
                    .color(palette.accent),
            ),
        )
        .children((!host.log().is_empty()).then(|| log(host, palette)));
    page(column)
}

fn log(host: &HostModel, palette: &Palette) -> Div {
    div()
        .w_full()
        .p(px(12.))
        .rounded(px(10.))
        .bg(palette.card)
        .border_1()
        .border_color(palette.sep)
        .flex()
        .flex_col()
        .gap(px(2.))
        .font_family(palette.mono())
        .text_size(px(11.))
        .text_color(palette.text2)
        .children(
            host.log()
                .last(LINES)
                .map(|line| div().truncate().child(line.to_string())),
        )
}
