use gpui_kit::*;

use crate::help::HelpExt;
use crate::shell::engine_state::{EngineState, open_diagnostics};
use crate::widgets::brand_mark;
use crate::workspace::Workspace;

/// The app icon. A click opens Diagnostics; the engine's state shows in the status
/// bar.
pub fn render(handle: &Entity<Workspace>, engine: &EngineState) -> Stateful<Div> {
    div()
        .id("rail-app-icon")
        .cursor_pointer()
        .on_click(open_diagnostics(handle))
        .relative()
        .flex_shrink_0()
        .mb(px(6.))
        .child(brand_mark(px(40.)))
        .help(engine.help())
}
