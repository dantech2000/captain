use captain_core::model::ComposeProject;
use captain_core::store::{ContainerGroup, GroupKey};
use gpui_kit::assets::IconName;
use gpui_kit::component::Icon;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use super::{container_row, project_actions};
use crate::icons::{CaptainIcon, cap_icon};
use crate::theme::Palette;
use crate::workspace::Workspace;

/// One Compose project, one Kubernetes namespace, or the loose containers, as a
/// rounded card. A click on the header folds the card. A Compose card also shows the
/// project's folder and service count, and has project actions. `project` is the Compose model of the
/// card's project, rebuilt from all its containers, not only the visible ones.
pub fn render(
    group: ContainerGroup,
    project: Option<&ComposeProject>,
    handle: &Entity<Workspace>,
    workspace: &Workspace,
    palette: &Palette,
) -> impl IntoElement {
    let (name, color, icon) = match &group.key {
        GroupKey::Project(name) => (
            name.clone(),
            palette.project_color(name),
            CaptainIcon::Stack,
        ),
        GroupKey::Namespace(name) => (
            name.clone(),
            palette.project_color(name),
            CaptainIcon::Cluster,
        ),
        GroupKey::Standalone => (
            "Loose containers".to_string(),
            palette.gray,
            CaptainIcon::Container,
        ),
    };
    let summary = match (&group.key, project) {
        (GroupKey::Project(_), Some(project)) => format!(
            "{} · {} of {} running",
            project.services_label(),
            project.active_count(),
            project.container_count()
        ),
        (GroupKey::Project(_), None) => format!(
            "{} of {} running",
            group.running_count(),
            group.containers.len()
        ),
        (GroupKey::Namespace(_), _) => format!(
            "Kubernetes namespace · {} of {} running",
            group.running_count(),
            group.containers.len()
        ),
        (GroupKey::Standalone, _) if group.containers.len() == 1 => "1 container".to_string(),
        (GroupKey::Standalone, _) => format!("{} containers", group.containers.len()),
    };

    let collapsed = workspace.is_collapsed(&group.key);
    let toggle = {
        let handle = handle.clone();
        let key = group.key.clone();
        move |_: &ClickEvent, _: &mut Window, cx: &mut App| {
            handle.update(cx, |workspace, cx| {
                workspace.toggle_collapsed(key.clone(), cx)
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
        .child(cap_icon(icon, px(20.), color))
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
        .when(group.project().is_some(), |this| {
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
