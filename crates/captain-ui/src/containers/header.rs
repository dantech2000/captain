use captain_core::store::ContainerFilter;
use gpui_kit::assets::IconName;
use gpui_kit::component::checkbox::Checkbox;
use gpui_kit::component::{Icon, Sizable};
use gpui_kit::*;

use crate::help::HelpExt;
use crate::theme::Palette;
use crate::widgets::{Segment, drag_region, segmented};
use crate::workspace::Workspace;

/// The page title, counts, the project chip, and the filter. The empty parts drag
/// the window.
pub fn render(
    handle: &Entity<Workspace>,
    workspace: &Workspace,
    palette: &Palette,
) -> impl IntoElement {
    let store = workspace.store();
    let (running, total) = workspace.shown_counts();
    let stopped = total - running;
    let projects = store.projects().len();
    let summary = match projects {
        0 => format!("{running} running · {stopped} stopped"),
        1 => format!("{running} running · {stopped} stopped · 1 project"),
        n => format!("{running} running · {stopped} stopped · {n} projects"),
    };

    let segments = ContainerFilter::ALL
        .into_iter()
        .map(|filter| {
            let handle = handle.clone();
            Segment {
                label: filter.label().into(),
                selected: workspace.filter() == filter,
                help: match filter {
                    ContainerFilter::All => "Show all containers.",
                    ContainerFilter::Running => {
                        "Show only running, paused, and restarting containers."
                    }
                    ContainerFilter::Stopped => "Show only stopped and created containers.",
                }
                .into(),
                on_click: Box::new(move |_, cx| {
                    handle.update(cx, |workspace, cx| workspace.set_filter(filter, cx));
                }),
            }
        })
        .collect();

    drag_region("containers-header")
        .flex_shrink_0()
        .flex()
        .items_center()
        .gap(px(12.))
        .px(px(24.))
        .pt(px(18.))
        .pb(px(14.))
        .child(
            div()
                .flex_1()
                .flex()
                .flex_col()
                .gap(px(2.))
                .child(
                    div()
                        .text_size(px(22.))
                        .font_weight(FontWeight::BOLD)
                        .child("Containers"),
                )
                .child(
                    div()
                        .text_size(px(12.))
                        .text_color(palette.text2)
                        .child(summary),
                ),
        )
        .children(
            (store.kubernetes_count() > 0).then(|| kubernetes_toggle(handle, workspace, palette)),
        )
        .children(
            workspace
                .project_filter()
                .map(|project| project_chip(project, handle, palette)),
        )
        .child(
            segmented("filter", segments, palette)
                .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation()),
        )
}

/// "Show Kubernetes containers". It shows only while the engine has pod containers.
/// The label is drawn here, smaller than gpui-kit's, and a click anywhere on the row
/// toggles it.
fn kubernetes_toggle(
    handle: &Entity<Workspace>,
    workspace: &Workspace,
    palette: &Palette,
) -> impl IntoElement {
    let handle = handle.clone();
    let checked = workspace.show_kubernetes();
    div()
        .id("show-kubernetes-help")
        .flex()
        .items_center()
        .gap(px(6.))
        .cursor_pointer()
        .text_size(px(12.))
        .text_color(palette.text2)
        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
        .on_click(move |_, _, cx| {
            handle.update(cx, |workspace, cx| {
                workspace.set_show_kubernetes(!checked, cx)
            });
        })
        .child(Checkbox::new("show-kubernetes").small().checked(checked))
        .child("Show Kubernetes containers")
        .help("Show the containers that Kubernetes runs for its pods, one card per namespace.")
}

/// "Project: shop ✕". A click clears the project filter.
fn project_chip(project: &str, handle: &Entity<Workspace>, palette: &Palette) -> Stateful<Div> {
    let handle = handle.clone();
    let hover = palette.accent.alpha(if palette.dark { 0.26 } else { 0.18 });
    div()
        .id("project-filter")
        .h(px(26.))
        .px(px(10.))
        .flex()
        .items_center()
        .gap(px(6.))
        .rounded(px(7.))
        .bg(palette.tint(palette.accent))
        .text_color(palette.link)
        .text_size(px(12.))
        .font_weight(FontWeight::MEDIUM)
        .cursor_pointer()
        .hover(move |style| style.bg(hover))
        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
        .on_click(move |_, _, cx| {
            handle.update(cx, |workspace, cx| workspace.clear_project_filter(cx));
        })
        .child(format!("Project: {project}"))
        .child(Icon::new(IconName::Close).size(px(11.)))
        .help(format!(
            "Show all containers again, not only those of {project}."
        ))
}
