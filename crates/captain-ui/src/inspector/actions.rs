use captain_core::model::{Container, ContainerAction, ContainerState};
use gpui_kit::assets::IconName;
use gpui_kit::component::Icon;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use super::delete_dialog;
use crate::help::HelpExt;
use crate::theme::Palette;
use crate::workspace::Workspace;

/// Start or Stop, Pause or Resume, Restart, Browser, and Delete, as five equal buttons.
/// Delete asks first, and offers "Stop and delete" for a live container.
pub fn render(
    container: &Container,
    handle: &Entity<Workspace>,
    workspace: &Workspace,
    palette: &Palette,
) -> impl IntoElement {
    let active = container.state.is_active();
    let pending = workspace.is_pending(&container.id);
    let port = container.published_ports().first().copied();
    let toggle = ContainerAction::toggle_for(container.state);
    let pause = ContainerAction::pause_toggle_for(container.state);

    let name = &container.name;
    let action = |action: ContainerAction, icon: IconName, enabled: bool| {
        let handle = handle.clone();
        let id = container.id.clone();
        let help = match action {
            ContainerAction::Start => format!("Start {name}."),
            ContainerAction::Stop => format!("Stop {name}. Its files and volumes stay."),
            ContainerAction::Pause => format!("Pause {name}. Its processes freeze, not stop."),
            ContainerAction::Unpause => format!("Resume the frozen processes of {name}."),
            _ => format!("Stop and start {name} again."),
        };
        button(
            action.label(),
            icon,
            help,
            palette.text,
            enabled && !pending,
            palette,
            move |_, cx| {
                handle.update(cx, |workspace, cx| {
                    workspace.run_action(id.clone(), action, cx)
                });
            },
        )
    };
    let browser = button(
        "Browser",
        IconName::ArrowUpRight,
        match port {
            Some(port) => format!("Open http://localhost:{port} in your browser."),
            None => format!("{name} publishes no port to open."),
        },
        palette.accent,
        port.is_some(),
        palette,
        move |_, cx| {
            if let Some(port) = port {
                cx.open_url(&format!("http://localhost:{port}"));
            }
        },
    );
    let delete = {
        let handle = handle.clone();
        let container = container.clone();
        button(
            ContainerAction::Remove.label(),
            IconName::Trash,
            format!("Delete {name}. Captain asks first."),
            palette.red,
            !pending,
            palette,
            move |window, cx| delete_dialog::open(&container, handle.clone(), window, cx),
        )
    };

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
            true,
        ))
        .child(match pause {
            Some(ContainerAction::Unpause) => {
                action(ContainerAction::Unpause, IconName::Play, true)
            }
            _ => action(ContainerAction::Pause, IconName::Pause, pause.is_some()),
        })
        .child(action(
            ContainerAction::Restart,
            IconName::RotateCw,
            container.state == ContainerState::Running,
        ))
        .child(browser)
        .child(delete)
}

fn button(
    label: &'static str,
    icon: IconName,
    help: String,
    color: Hsla,
    enabled: bool,
    palette: &Palette,
    on_click: impl Fn(&mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let hover = palette.nav_selected;
    div()
        .id(label)
        .flex_1()
        .min_w_0()
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
        .text_color(palette.readable(color))
        .when(!enabled, |this| this.opacity(0.4))
        .when(enabled, |this| {
            this.cursor_pointer()
                .hover(move |style| style.bg(hover))
                .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                .on_click(move |_, window, cx| on_click(window, cx))
        })
        .child(Icon::new(icon).size(px(16.)))
        .child(label)
        .help(help)
}
