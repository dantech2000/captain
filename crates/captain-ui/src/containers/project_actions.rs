use captain_core::model::{ContainerAction, ProjectAction};
use captain_core::store::ContainerGroup;
use gpui_kit::assets::IconName;
use gpui_kit::component::spinner::Spinner;
use gpui_kit::component::{Icon, Sizable};
use gpui_kit::*;

use super::down_dialog;
use crate::help::HelpExt;
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
    let count = group.containers.len();
    let mut actions = vec![(
        ProjectAction::Up,
        IconName::Play,
        format!("Create and start the services of {name} (docker compose up)."),
    )];
    if group.running_count() > 0 {
        actions.push((
            ProjectAction::Stop,
            IconName::Square,
            format!("Stop all {count} services of {name}. The containers stay."),
        ));
    }
    actions.extend([
        (
            ProjectAction::Restart,
            IconName::RotateCw,
            format!("Restart all {count} services of {name}."),
        ),
        (
            ProjectAction::Pull,
            IconName::Download,
            format!("Pull newer images for the services of {name}."),
        ),
        (
            ProjectAction::Down,
            IconName::PowerOff,
            format!("Stop and remove the {count} containers of {name}. Volumes and images stay."),
        ),
    ]);
    actions
        .into_iter()
        .map(|(action, icon, help)| {
            let handle = handle.clone();
            let project = name.to_string();
            icon_button(
                SharedString::from(format!("{}-{name}", action.label())),
                icon,
                help,
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
    let count = all.len();
    let toggle = if active.is_empty() {
        (
            "Start all",
            IconName::Play,
            format!("Start all {count} containers of {name}."),
            ContainerAction::Start,
            all.clone(),
        )
    } else {
        (
            "Stop all",
            IconName::Square,
            format!("Stop the {} running containers of {name}.", active.len()),
            ContainerAction::Stop,
            active,
        )
    };
    let restart = (
        "Restart all",
        IconName::RotateCw,
        format!("Restart all {count} containers of {name}."),
        ContainerAction::Restart,
        all,
    );
    [toggle, restart]
        .map(|(label, icon, help, action, ids)| {
            let handle = handle.clone();
            icon_button(
                SharedString::from(format!("{label}-{name}")),
                icon,
                help,
                palette,
                move |_, _, cx| {
                    cx.stop_propagation();
                    handle.update(cx, |workspace, cx| {
                        workspace.run_actions(ids.clone(), action, cx)
                    });
                },
            )
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
        .help("Install Docker Compose for Up, Down, and Pull.")
}
