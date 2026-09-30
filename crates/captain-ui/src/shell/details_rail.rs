//! The thin strip that stands in for the hidden details panel.

use gpui_kit::assets::IconName;
use gpui_kit::*;

use crate::theme::Palette;
use crate::widgets::icon_button;
use crate::workspace::Workspace;

/// A narrow column at the right edge with one button that shows the details of
/// `name` again.
pub fn render(workspace: &Entity<Workspace>, name: &str, palette: &Palette) -> Div {
    let workspace = workspace.clone();
    div()
        .w(px(40.))
        .h_full()
        .flex_shrink_0()
        .flex()
        .flex_col()
        .items_center()
        .pt(px(16.))
        .border_l_1()
        .border_color(palette.sep)
        .bg(palette.panel)
        .child(icon_button(
            "details-show",
            IconName::PanelRightOpen,
            format!("Show the details of {name}."),
            palette,
            move |_, _, cx| {
                workspace.update(cx, |workspace, cx| workspace.set_details_hidden(false, cx));
            },
        ))
}
