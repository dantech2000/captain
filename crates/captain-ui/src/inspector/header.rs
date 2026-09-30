use captain_core::model::Container;
use gpui_kit::assets::IconName;
use gpui_kit::*;

use crate::icons::{CaptainIcon, cap_icon};
use crate::theme::Palette;
use crate::widgets::{icon_button, pill};

/// The container icon, name, state, image, uptime, and project, and a button that
/// hides the panel.
pub fn render(
    container: &Container,
    palette: &Palette,
    hide: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let color = container
        .compose_project
        .as_deref()
        .map_or(palette.gray, |p| palette.project_color(p));
    let state_color = palette.container_state(container.state);
    let project = match &container.compose_project {
        Some(project) => format!("project {project}"),
        None => "standalone".to_string(),
    };

    div()
        .flex()
        .items_start()
        .gap(px(12.))
        .child(
            div()
                .size(px(44.))
                .flex_shrink_0()
                .rounded(px(12.))
                .bg(palette.tint(color))
                .flex()
                .items_center()
                .justify_center()
                .child(cap_icon(CaptainIcon::Container, px(22.), color)),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .gap(px(4.))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .child(
                            div()
                                .text_size(px(18.))
                                .font_weight(FontWeight::BOLD)
                                .truncate()
                                .child(container.display_name()),
                        )
                        .child(pill(
                            container.state.label(),
                            palette.readable(state_color),
                            palette.tint(state_color),
                        )),
                )
                .child(
                    div()
                        .font_family(palette.mono())
                        .text_size(px(12.))
                        .text_color(palette.text2)
                        .truncate()
                        .child(container.image.clone()),
                )
                .child(
                    div()
                        .text_size(px(11.))
                        .text_color(palette.text3)
                        .child(format!("{} · {project}", container.status)),
                ),
        )
        .child(icon_button(
            "details-hide",
            IconName::PanelRightClose,
            "Hide the details panel. Select a container, or use the button at the right edge, to show it again.",
            palette,
            hide,
        ))
}
