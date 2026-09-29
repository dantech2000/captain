mod brand;
mod engine_card;
mod nav;
mod projects;

pub use projects::project_badge;

use gpui_kit::*;

use crate::theme::Palette;
use crate::widgets::drag_region;
use crate::workspace::Workspace;

pub fn render(workspace: &Workspace, palette: &Palette) -> impl IntoElement {
    div()
        .w(px(244.))
        .h_full()
        .flex_shrink_0()
        .flex()
        .flex_col()
        .gap(px(2.))
        .px(px(12.))
        .pb(px(14.))
        .bg(palette.side)
        .border_r_1()
        .border_color(palette.sep)
        .child(drag_region("sidebar-drag").h(px(48.)).flex_shrink_0())
        .child(brand::render(workspace.connection(), palette))
        .child(nav::render(workspace.store().len(), palette))
        .child(projects::render(workspace.store(), palette))
        .child(div().flex_1())
        .child(engine_card::render(workspace, palette))
}
