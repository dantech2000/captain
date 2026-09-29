use captain_core::store::ContainerGroup;
use gpui_kit::assets::IconName;
use gpui_kit::component::Icon;
use gpui_kit::*;

use super::container_row;
use crate::shell::project_badge;
use crate::theme::Palette;
use crate::workspace::Workspace;

/// One Compose project, or the standalone containers, as a rounded card.
pub fn render(
    group: ContainerGroup,
    handle: &Entity<Workspace>,
    workspace: &Workspace,
    palette: &Palette,
) -> impl IntoElement {
    let (name, color) = match &group.project {
        Some(name) => (name.clone(), palette.project_color(name)),
        None => ("Standalone".to_string(), palette.gray),
    };
    let summary = match group.project {
        Some(_) => format!(
            "{} of {} running",
            group.running_count(),
            group.containers.len()
        ),
        None if group.containers.len() == 1 => "1 container".to_string(),
        None => format!("{} containers", group.containers.len()),
    };

    let header = div()
        .h(px(38.))
        .px(px(10.))
        .flex()
        .items_center()
        .gap(px(10.))
        .child(
            Icon::new(IconName::ChevronDown)
                .size(px(12.))
                .text_color(palette.text3),
        )
        .child(project_badge(&name, color, px(20.), palette))
        .child(div().font_weight(FontWeight::SEMIBOLD).child(name))
        .child(div().flex_1())
        .child(
            div()
                .text_size(px(11.))
                .text_color(palette.text2)
                .child(summary),
        );

    div()
        .flex()
        .flex_col()
        .p(px(4.))
        .rounded(px(12.))
        .bg(palette.group)
        .border_1()
        .border_color(palette.sep)
        .child(header)
        .children(
            group
                .containers
                .iter()
                .map(|container| container_row::render(container, handle, workspace, palette)),
        )
}
