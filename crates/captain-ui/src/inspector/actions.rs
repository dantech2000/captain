use captain_core::model::{Container, ContainerAction};
use gpui_kit::assets::IconName;
use gpui_kit::component::Icon;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use crate::theme::Palette;
use crate::workspace::Workspace;

/// Start or Stop, Restart, Browser, and Delete. Delete works only on a stopped container.
pub fn render(
    container: &Container,
    handle: &Entity<Workspace>,
    workspace: &Workspace,
    palette: &Palette,
) -> impl IntoElement {
    let active = container.state.is_active();
    let pending = workspace.is_pending(&container.id);
    let port = container.published_ports().first().copied();
    let toggle = if active {
        ContainerAction::Stop
    } else {
        ContainerAction::Start
    };

    let action = |action: ContainerAction, icon: IconName, color: Hsla, enabled: bool| {
        let handle = handle.clone();
        let id = container.id.clone();
        button(
            action.label(),
            icon,
            color,
            enabled && !pending,
            palette,
            move |cx| {
                handle.update(cx, |workspace, cx| {
                    workspace.run_action(id.clone(), action, cx)
                });
            },
        )
    };
    let browser = button(
        "Browser",
        IconName::ArrowUpRight,
        palette.accent,
        port.is_some(),
        palette,
        move |cx| {
            if let Some(port) = port {
                cx.open_url(&format!("http://localhost:{port}"));
            }
        },
    );

    div()
        .flex()
        .flex_col()
        .gap(px(8.))
        .child(
            div()
                .flex()
                .gap(px(8.))
                .child(action(
                    toggle,
                    if active {
                        IconName::Square
                    } else {
                        IconName::Play
                    },
                    palette.text,
                    true,
                ))
                .child(action(
                    ContainerAction::Restart,
                    IconName::RotateCw,
                    palette.text,
                    active,
                ))
                .child(browser)
                .child(action(
                    ContainerAction::Remove,
                    IconName::Trash,
                    palette.red,
                    !active,
                )),
        )
        .children(workspace.action_error().map(|error| {
            div()
                .text_size(px(11.))
                .text_color(palette.red)
                .child(error.to_string())
        }))
}

fn button(
    label: &'static str,
    icon: IconName,
    color: Hsla,
    enabled: bool,
    palette: &Palette,
    on_click: impl Fn(&mut App) + 'static,
) -> Stateful<Div> {
    let hover = palette.nav_selected;
    div()
        .id(label)
        .flex_1()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(5.))
        .pt(px(9.))
        .pb(px(7.))
        .rounded(px(10.))
        .border_1()
        .border_color(palette.sep)
        .bg(palette.button)
        .text_size(px(11.))
        .text_color(color)
        .when(!enabled, |this| this.opacity(0.4))
        .when(enabled, |this| {
            this.cursor_pointer()
                .hover(move |style| style.bg(hover))
                .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                .on_click(move |_, _, cx| on_click(cx))
        })
        .child(Icon::new(icon).size(px(16.)))
        .child(label)
}
