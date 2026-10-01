use gpui_kit::*;

use crate::help::HelpExt;
use crate::shell::engine_state::{EngineState, open_diagnostics};
use crate::theme::Palette;
use crate::widgets::brand_mark;
use crate::workspace::Workspace;

/// The app icon with a dot in the engine's state color, so the state shows while
/// the projects list is hidden. A click opens Diagnostics.
pub fn render(
    handle: &Entity<Workspace>,
    engine: &EngineState,
    palette: &Palette,
) -> Stateful<Div> {
    div()
        .id("rail-app-icon")
        .cursor_pointer()
        .on_click(open_diagnostics(handle))
        .relative()
        .flex_shrink_0()
        .mb(px(6.))
        .child(brand_mark(px(40.)))
        .child(
            div()
                .absolute()
                .right(px(3.))
                .bottom(px(3.))
                .size(px(11.))
                .rounded_full()
                .border_2()
                .border_color(palette.rail)
                .bg(engine.color),
        )
        .help(engine.help())
}
