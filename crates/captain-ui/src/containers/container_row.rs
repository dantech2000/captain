use captain_core::format::{bytes_label, percent_label};
use captain_core::model::{Container, ContainerAction, ContainerState, Health};
use gpui_kit::assets::IconName;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use crate::theme::Palette;
use crate::widgets::{icon_button, pill, scales, sparkline, status_dot};
use crate::workspace::Workspace;

pub const PORTS_WIDTH: f32 = 150.;
pub const CPU_WIDTH: f32 = 112.;
pub const MEMORY_WIDTH: f32 = 64.;
pub const TRAILING_WIDTH: f32 = 92.;

/// One container: state, name, image, ports, CPU, memory, and uptime or actions.
pub fn render(
    container: &Container,
    handle: &Entity<Workspace>,
    workspace: &Workspace,
    palette: &Palette,
) -> impl IntoElement {
    let selected = workspace.selected().is_some_and(|s| s.id == container.id);
    let history = workspace.stats().get(&container.id);
    let running = container.state == ContainerState::Running;
    let cpu = history
        .and_then(|h| h.latest())
        .map(|s| percent_label(s.cpu_percent))
        .unwrap_or_else(|| "—".into());
    let memory = history
        .and_then(|h| h.latest())
        .map(|s| bytes_label(s.memory_bytes))
        .unwrap_or_else(|| "—".into());
    let series = history.map(|h| h.cpu_series()).unwrap_or_default();
    let id = container.id.clone();
    let select = handle.clone();

    div()
        .id(SharedString::from(format!("row-{}", container.id)))
        .h(px(50.))
        .px(px(12.))
        .flex()
        .items_center()
        .gap(px(12.))
        .rounded(px(9.))
        .cursor_pointer()
        .when(selected, |row| {
            row.bg(palette.accent.alpha(if palette.dark { 0.15 } else { 0.08 }))
                .border_1()
                .border_color(palette.accent.alpha(0.35))
        })
        .when(!selected, |row| row.hover(|style| style.bg(palette.group)))
        .on_click(move |_, _, cx| {
            select.update(cx, |workspace, cx| workspace.select(id.clone(), cx));
        })
        .child(name_cell(container, palette))
        .child(ports_cell(container, palette))
        .child(
            div()
                .w(px(CPU_WIDTH))
                .flex_shrink_0()
                .flex()
                .items_center()
                .gap(px(8.))
                .child(
                    sparkline(
                        series,
                        scales::CPU,
                        if running {
                            palette.accent
                        } else {
                            palette.text3
                        },
                        None,
                    )
                    .w(px(58.))
                    .h(px(20.)),
                )
                .child(
                    div()
                        .text_size(px(11.))
                        .text_color(palette.text2)
                        .child(cpu),
                ),
        )
        .child(
            div()
                .w(px(MEMORY_WIDTH))
                .flex_shrink_0()
                .text_right()
                .text_size(px(12.))
                .text_color(palette.text2)
                .child(memory),
        )
        .child(trailing_cell(
            container, selected, handle, workspace, palette,
        ))
}

fn name_cell(container: &Container, palette: &Palette) -> Div {
    let (color, glow) = match container.state {
        ContainerState::Running => (palette.green, true),
        ContainerState::Paused | ContainerState::Restarting => (palette.orange, false),
        ContainerState::Dead => (palette.red, false),
        _ => (palette.gray, false),
    };
    let (image, tag) = container.image_name_and_tag();
    let health = container.health.map(|health| {
        let color = match health {
            Health::Healthy => palette.green,
            Health::Starting => palette.orange,
            Health::Unhealthy => palette.red,
        };
        pill(health.label(), color, palette.tint(color))
    });
    // The Compose service, when the container name does not already say it.
    let service = container
        .compose
        .service
        .clone()
        .filter(|service| *service != container.name);

    div()
        .flex_1()
        .min_w_0()
        .flex()
        .items_center()
        .gap(px(12.))
        .child(status_dot(color, glow, palette))
        .child(
            div()
                .min_w_0()
                .flex()
                .flex_col()
                .gap(px(3.))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.))
                        .child(
                            div()
                                .font_weight(FontWeight::SEMIBOLD)
                                .truncate()
                                .child(container.name.clone()),
                        )
                        .children(service.map(|service| {
                            div()
                                .flex_shrink_0()
                                .text_size(px(11.))
                                .text_color(palette.text3)
                                .child(service)
                        }))
                        .children(health),
                )
                .child(
                    div()
                        .flex()
                        .font_family(palette.mono())
                        .text_size(px(11.))
                        .text_color(palette.text2)
                        .truncate()
                        .child(image.to_string())
                        .when(!tag.is_empty(), |this| {
                            this.child(div().text_color(palette.text3).child(format!(":{tag}")))
                        }),
                ),
        )
}

fn ports_cell(container: &Container, palette: &Palette) -> Div {
    let ports = container.published_ports();
    let note = if container.state.is_active() {
        "Internal only"
    } else {
        "Not running"
    };
    div()
        .w(px(PORTS_WIDTH))
        .flex_shrink_0()
        .flex()
        .gap(px(4.))
        .overflow_hidden()
        .when(ports.is_empty(), |this| {
            this.child(
                div()
                    .text_size(px(11.))
                    .text_color(palette.text3)
                    .child(note),
            )
        })
        .children(ports.into_iter().take(2).map(|port| {
            div()
                .id(SharedString::from(format!("port-{}-{port}", container.id)))
                .h(px(22.))
                .px(px(8.))
                .flex()
                .items_center()
                .gap(px(4.))
                .rounded(px(6.))
                .bg(palette.accent.alpha(if palette.dark { 0.12 } else { 0.08 }))
                .text_color(palette.accent)
                .font_family(palette.mono())
                .text_size(px(11.))
                .cursor_pointer()
                .on_click(move |_, _, cx| {
                    cx.stop_propagation();
                    cx.open_url(&format!("http://localhost:{port}"));
                })
                .child(format!(":{port}"))
                .child(gpui_kit::component::Icon::new(IconName::ArrowUpRight).size(px(10.)))
        }))
}

fn trailing_cell(
    container: &Container,
    selected: bool,
    handle: &Entity<Workspace>,
    workspace: &Workspace,
    palette: &Palette,
) -> Div {
    let cell = div()
        .w(px(TRAILING_WIDTH))
        .flex_shrink_0()
        .flex()
        .justify_end()
        .items_center()
        .gap(px(4.));
    if !selected || workspace.is_pending(&container.id) {
        let label = if workspace.is_pending(&container.id) {
            "Working...".to_string()
        } else {
            container.uptime_label()
        };
        return cell.child(
            div()
                .text_size(px(12.))
                .text_color(palette.text3)
                .truncate()
                .child(label),
        );
    }

    let toggle = ContainerAction::toggle_for(container.state);
    let toggle_icon = match toggle {
        ContainerAction::Stop => IconName::Square,
        _ => IconName::Play,
    };
    let pause = ContainerAction::pause_toggle_for(container.state).map(|action| match action {
        ContainerAction::Pause => (action, IconName::Pause),
        _ => (action, IconName::Play),
    });
    let action_button = |action: ContainerAction, icon: IconName| {
        let handle = handle.clone();
        let id = container.id.clone();
        icon_button(
            SharedString::from(format!("{}-{}", action.label(), container.id)),
            icon,
            palette,
            move |_, _, cx| {
                cx.stop_propagation();
                handle.update(cx, |workspace, cx| {
                    workspace.run_action(id.clone(), action, cx)
                });
            },
        )
    };
    cell.child(action_button(toggle, toggle_icon))
        .children(pause.map(|(action, icon)| action_button(action, icon)))
        .child(action_button(ContainerAction::Restart, IconName::RotateCw))
}
