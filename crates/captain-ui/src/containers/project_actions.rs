use captain_core::model::{ContainerAction, ProjectAction};
use captain_core::store::ContainerGroup;
use gpui_kit::assets::IconName;
use gpui_kit::component::spinner::Spinner;
use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::component::{Icon, Sizable};
use gpui_kit::*;

use super::down_dialog;
use crate::theme::Palette;
use crate::widgets::icon_button;
use crate::workspace::Workspace;

/// The buttons on a Compose card header. With `docker compose`: Up, Stop (while any
/// container runs), Restart, Pull, and Down. Without it: the engine's Start all or
/// Stop all, and Restart all, with a hint. A running command shows a spinner instead.
pub fn render(
    name: &str,
    group: &ContainerGroup,
    handle: &Entity<Workspace>,
    workspace: &Workspace,
    palette: &Palette,
) -> Div {
    let row = div().flex().items_center().gap(px(4.));
    if let Some(action) = workspace.project_pending(name) {
        return row
            .gap(px(6.))
            .text_size(px(11.))
            .text_color(palette.text2)
            .child(Spinner::new().xsmall().color(palette.text2))
            .child(action.progress_label());
    }
    if workspace.has_project_runner() {
        row.children(compose_buttons(name, group, handle, palette))
    } else {
        row.children(engine_buttons(name, group, handle, palette))
            .child(hint(name, palette))
    }
}

fn compose_buttons(
    name: &str,
    group: &ContainerGroup,
    handle: &Entity<Workspace>,
    palette: &Palette,
) -> Vec<Stateful<Div>> {
    let mut actions = vec![(ProjectAction::Up, IconName::Play, "Up: create and start")];
    if group.running_count() > 0 {
        actions.push((ProjectAction::Stop, IconName::Square, "Stop all services"));
    }
    actions.extend([
        (
            ProjectAction::Restart,
            IconName::RotateCw,
            "Restart all services",
        ),
        (ProjectAction::Pull, IconName::Download, "Pull the images"),
        (
            ProjectAction::Down,
            IconName::PowerOff,
            "Down: stop and remove",
        ),
    ]);
    actions
        .into_iter()
        .map(|(action, icon, tip)| {
            let handle = handle.clone();
            let project = name.to_string();
            icon_button(
                SharedString::from(format!("{}-{name}", action.label())),
                icon,
                palette,
                move |_, window, cx| {
                    cx.stop_propagation();
                    if action == ProjectAction::Down {
                        down_dialog::open(project.clone(), handle.clone(), window, cx);
                    } else {
                        handle.update(cx, |workspace, cx| {
                            workspace.run_project_action(project.clone(), action, cx)
                        });
                    }
                },
            )
            .tooltip(move |window, cx| Tooltip::new(tip).build(window, cx))
        })
        .collect()
}

/// Start all (when nothing runs) or Stop all, and Restart all, through the engine.
fn engine_buttons(
    name: &str,
    group: &ContainerGroup,
    handle: &Entity<Workspace>,
    palette: &Palette,
) -> Vec<Stateful<Div>> {
    let all: Vec<String> = group.containers.iter().map(|c| c.id.clone()).collect();
    let active: Vec<String> = group
        .containers
        .iter()
        .filter(|c| c.state.is_active())
        .map(|c| c.id.clone())
        .collect();
    let toggle = if active.is_empty() {
        (
            "Start all",
            IconName::Play,
            ContainerAction::Start,
            all.clone(),
        )
    } else {
        ("Stop all", IconName::Square, ContainerAction::Stop, active)
    };
    let restart = (
        "Restart all",
        IconName::RotateCw,
        ContainerAction::Restart,
        all,
    );
    [toggle, restart]
        .map(|(label, icon, action, ids)| {
            let handle = handle.clone();
            icon_button(
                SharedString::from(format!("{label}-{name}")),
                icon,
                palette,
                move |_, _, cx| {
                    cx.stop_propagation();
                    handle.update(cx, |workspace, cx| {
                        workspace.run_actions(ids.clone(), action, cx)
                    });
                },
            )
            .tooltip(move |window, cx| Tooltip::new(label).build(window, cx))
        })
        .into()
}

/// Why Up, Down, and Pull are missing.
fn hint(name: &str, palette: &Palette) -> Stateful<Div> {
    div()
        .id(SharedString::from(format!("compose-hint-{name}")))
        .size(px(26.))
        .flex()
        .items_center()
        .justify_center()
        .text_color(palette.text3)
        .child(Icon::new(IconName::Info).size(px(13.)))
        .tooltip(|window, cx| {
            Tooltip::new("Install Docker Compose for Up, Down, and Pull.").build(window, cx)
        })
}
