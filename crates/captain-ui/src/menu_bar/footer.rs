//! The popover's bottom buttons: Open Captain, Float logs, and Stop all.

use captain_core::model::ContainerAction;
use gpui_kit::*;

use super::controls::{Tone, button};
use super::float_log::open_float_log;
use super::popover::close_popover;
use crate::theme::Palette;
use crate::workspace::Workspace;

pub fn render(
    float: Option<(String, String)>,
    active: Vec<String>,
    workspace: &Entity<Workspace>,
    open_captain: fn(&mut App),
    palette: &Palette,
) -> Div {
    let open = button(
        "popover-open-captain",
        "Open Captain",
        "Open the Captain window.",
        Tone::Plain,
        true,
        palette,
        move |_, cx| {
            close_popover(cx);
            open_captain(cx);
        },
    )
    .flex_1();
    let float = float.map(|(id, name)| {
        let help = format!("Open {name}'s log in a small window that stays on top.");
        let workspace = workspace.clone();
        button(
            "popover-float-logs",
            "Float logs".to_string(),
            help,
            Tone::Plain,
            true,
            palette,
            move |_, cx| {
                close_popover(cx);
                open_float_log(workspace.clone(), id.clone(), name.clone(), cx);
            },
        )
        .flex_1()
        .min_w_0()
    });
    let stop_all = (!active.is_empty()).then(|| {
        let workspace = workspace.clone();
        let help = format!(
            "Stop {}. Nothing is removed.",
            captain_core::model::count_label(active.len(), "running container")
        );
        button(
            "popover-stop-all",
            "Stop all",
            help,
            Tone::Quiet,
            true,
            palette,
            move |_, cx| {
                let ids = active.clone();
                workspace.update(cx, |workspace, cx| {
                    workspace.run_actions(ids, ContainerAction::Stop, cx)
                });
            },
        )
    });
    div()
        .flex()
        .gap(px(6.))
        .p(px(12.))
        .border_t_1()
        .border_color(palette.sep)
        .child(open)
        .children(float)
        .children(stop_all)
}
