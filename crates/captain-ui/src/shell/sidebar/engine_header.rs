use gpui_kit::*;

use crate::engine_host::HostSummary;
use crate::help::HelpExt;
use crate::shell::engine_state::EngineState;
use crate::theme::Palette;
use crate::workspace::Workspace;

/// The app name and one line of engine state: for example `Captain Engine ·
/// Running`. The app icon stands in the rail to the left. Its height never changes.
pub fn render(
    workspace: &Workspace,
    host: Option<&HostSummary>,
    palette: &Palette,
) -> Stateful<Div> {
    // The numbers live in the status line and the status bar; here only which
    // engine it is and its state.
    let engine = EngineState::of(workspace, host, palette);
    div()
        .id("sidebar-engine-header")
        .flex_shrink_0()
        .flex()
        .flex_col()
        .gap(px(2.))
        .px(px(4.))
        .pb(px(12.))
        .child(
            div()
                .text_size(px(15.))
                .font_weight(FontWeight::SEMIBOLD)
                .child("Captain"),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(6.))
                .text_size(px(11.))
                .text_color(palette.text2)
                .child(div().size(px(6.)).rounded_full().bg(engine.color))
                .child(div().min_w_0().truncate().child(engine.line())),
        )
        .help(engine.help())
}
