use captain_core::model::Container;
use captain_core::project_map::layout;
use gpui_kit::*;

use super::inputs::{services, titles};
use super::node::{self, Node};
use super::scale::Scale;
use super::{drawer, edges, editor, marks, overlay};
use crate::project::ProjectView;
use crate::project::group_info::service_name;
use crate::theme::Palette;
use crate::workspace::{Connection, Workspace};

/// The map of `containers` and, when there are any, their staged changes.
pub fn render(
    view: &ProjectView,
    containers: &[&Container],
    handle: &Entity<Workspace>,
    workspace: &Workspace,
    weak: &WeakEntity<ProjectView>,
    palette: &Palette,
) -> Div {
    let titles = titles(containers);
    let map = layout(&services(containers, &titles, &view.details));
    let scale = view.map.scale(map.width);
    let z = Scale(scale);
    let stats = workspace.stats();
    let selected = workspace
        .selected()
        .map(|c| c.id.clone())
        .filter(|_| workspace.card_open());

    let mut content: Vec<AnyElement> = Vec::new();
    for lane in &map.lanes {
        content.push(marks::lane(lane, z, palette).into_any_element());
    }
    content.push(marks::column_label("This Mac", map.pins_label, z, palette).into_any_element());
    if let Some(at) = map.volumes_label {
        content.push(marks::column_label("Volumes", at, z, palette).into_any_element());
    }
    for (port, id, rect) in &map.pins {
        let Some((container, title)) = containers.iter().zip(&titles).find(|(c, _)| &c.id == id)
        else {
            continue;
        };
        let private = container
            .ports
            .iter()
            .find(|p| p.public_port == Some(*port))
            .map_or(*port, |p| p.private_port);
        content.push(marks::pin(*port, private, title, *rect, z, weak, palette).into_any_element());
    }
    for volume in &map.volumes {
        let size = view.map.volume_sizes.get(&volume.key).copied();
        content.push(marks::volume(volume, size, z, palette).into_any_element());
    }
    for (container, title) in containers.iter().zip(&titles) {
        let Some(rect) = map.node(&container.id) else {
            continue;
        };
        let node = Node {
            container,
            title,
            detail: view.details.get(&container.id).map(|(_, detail)| detail),
            history: stats.get(&container.id),
            exits: view.recent_exits(&service_name(container)),
            staged: view.staged.has(&container.id),
            selected: selected.as_deref() == Some(container.id.as_str()),
            rect,
        };
        content.push(node::render(node, z, handle, weak, palette).into_any_element());
    }
    let cpus = match workspace.connection() {
        Connection::Connected(info) => info.cpus,
        _ => 1,
    };
    if let Some(draft) = &view.map.draft
        && let Some(rect) = map.node(&draft.id)
    {
        content.push(
            editor::render(draft, (rect, map.height), cpus, z, weak, palette).into_any_element(),
        );
    }

    let width = view.map.width.clone();
    let measure = {
        let weak = weak.clone();
        canvas(
            move |bounds, _, cx| {
                let measured = f32::from(bounds.size.width);
                if (width.get() - measured).abs() > 1. {
                    width.set(measured);
                    let weak = weak.clone();
                    cx.defer(move |cx| {
                        weak.update(cx, |_, cx| cx.notify()).ok();
                    });
                }
            },
            |_, _, _, _| {},
        )
        .absolute()
        .size_full()
    };
    let area = div()
        .flex_1()
        .min_h_0()
        .relative()
        .border_t_1()
        .border_color(palette.sep)
        .bg(palette.bg)
        .child(measure)
        .child(
            div().id("project-map").size_full().overflow_scroll().child(
                div()
                    .relative()
                    .w(z.px(map.width))
                    .h(z.px(map.height))
                    .child(
                        edges::edges(map.edges.clone(), z, palette)
                            .absolute()
                            .size_full(),
                    )
                    .children(content),
            ),
        )
        .child(overlay::legend(palette))
        .child(overlay::zoom(view.map.zoom, scale, weak, palette))
        .children(containers.is_empty().then(|| empty(palette)));

    let ids: Vec<String> = containers.iter().map(|c| c.id.clone()).collect();
    let changes: Vec<_> = view
        .staged
        .changes()
        .iter()
        .filter(|c| ids.contains(&c.container_id))
        .cloned()
        .collect();
    let busy = ids.iter().any(|id| view.map.applying.contains(id));
    div()
        .flex_1()
        .min_h_0()
        .flex()
        .flex_col()
        .child(area)
        .children((!changes.is_empty()).then(|| drawer::render(changes, ids, busy, weak, palette)))
}

/// The note in place of the map when the entry has no containers.
fn empty(palette: &Palette) -> Div {
    div()
        .absolute()
        .top(px(100.))
        .left_0()
        .right_0()
        .flex()
        .justify_center()
        .text_size(px(12.))
        .text_color(palette.text2)
        .child("No containers to map. Start the project to see its ports, networks, and volumes.")
}
