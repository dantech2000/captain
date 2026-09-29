use captain_core::model::ComposeProject;
use captain_core::store::ContainerGroup;
use gpui_kit::assets::IconName;
use gpui_kit::component::Icon;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use super::{container_row, project_actions};
use crate::shell::project_badge;
use crate::theme::Palette;
use crate::workspace::Workspace;

/// One Compose project, or the standalone containers, as a rounded card. A click on
/// the header folds the card. A Compose card also shows the project's folder and
/// service count, and has project actions. `project` is the Compose model of the
/// card's project, rebuilt from all its containers, not only the visible ones.
pub fn render(
    group: ContainerGroup,
    project: Option<&ComposeProject>,
    handle: &Entity<Workspace>,
    workspace: &Workspace,
    palette: &Palette,
) -> impl IntoElement {
    let (name, color) = match &group.project {
        Some(name) => (name.clone(), palette.project_color(name)),
        None => ("Standalone".to_string(), palette.gray),
    };
    let summary = match (&group.project, project) {
        (Some(_), Some(project)) => format!(
            "{} · {} of {} running",
            project.services_label(),
            project.active_count(),
            project.container_count()
        ),
        (Some(_), None) => format!(
            "{} of {} running",
            group.running_count(),
            group.containers.len()
        ),
        (None, _) if group.containers.len() == 1 => "1 container".to_string(),
        (None, _) => format!("{} containers", group.containers.len()),
    };

    let collapsed = workspace.is_collapsed(&group.project);
    let toggle = {
        let handle = handle.clone();
        let project = group.project.clone();
        move |_: &ClickEvent, _: &mut Window, cx: &mut App| {
            handle.update(cx, |workspace, cx| {
                workspace.toggle_collapsed(project.clone(), cx)
            });
        }
    };
    let working_dir = project.and_then(|p| p.short_working_dir(std::env::home_dir().as_deref()));
    let header = div()
        .id(SharedString::from(format!("card-{name}")))
        .h(px(38.))
        .px(px(10.))
        .flex()
        .items_center()
        .gap(px(10.))
        .cursor_pointer()
        .on_click(toggle)
        .child(
            Icon::new(if collapsed {
                IconName::ChevronRight
            } else {
                IconName::ChevronDown
            })
            .size(px(12.))
            .text_color(palette.text3),
        )
        .child(project_badge(&name, color, px(20.), palette))
        .child(
            div()
                .flex_shrink_0()
                .font_weight(FontWeight::SEMIBOLD)
                .child(name.clone()),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .font_family(palette.mono())
                .text_size(px(11.))
                .text_color(palette.text3)
                .truncate()
                .children(working_dir),
        )
        .child(
            div()
                .flex_shrink_0()
                .text_size(px(11.))
                .text_color(palette.text2)
                .child(summary),
        )
        .when(group.project.is_some(), |this| {
            this.child(project_actions::render(
                &name, &group, handle, workspace, palette,
            ))
        });

    div()
        .flex()
        .flex_col()
        .p(px(4.))
        .rounded(px(12.))
        .bg(palette.group)
        .border_1()
        .border_color(palette.sep)
        .child(header)
        .when(!collapsed, |this| {
            this.children(
                group
                    .containers
                    .iter()
                    .map(|container| container_row::render(container, handle, workspace, palette)),
            )
        })
}
