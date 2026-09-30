use captain_core::model::Container;
use captain_core::store::GroupKey;
use gpui_kit::*;

use super::group_info::{is_sandbox, service_name};
use super::service_card::{self, Card};
use super::{ProjectView, header, open_row, tasks_card};
use crate::engine_host::host_screen;
use crate::theme::Palette;
use crate::workspace::Connection;

/// Lays out the Project page. While Captain Engine is not running, its screens
/// take the page, as on the Containers page.
pub fn render(view: &ProjectView, cx: &mut Context<ProjectView>) -> AnyElement {
    let palette = Palette::of(cx);
    if let Some(screen) = view
        .host
        .as_ref()
        .and_then(|host| host_screen(host, &palette, cx))
    {
        return screen;
    }
    let handle = view.workspace.clone();
    let weak = cx.entity().downgrade();
    let workspace = view.workspace.read(cx);
    let Some(key) = workspace.focus().cloned() else {
        return div().into_any_element();
    };
    if let Connection::Failed(error) = workspace.connection() {
        return div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .text_color(palette.text2)
            .child(format!("The engine does not answer: {error}"))
            .into_any_element();
    }
    let group = workspace.focused_group();
    let containers: Vec<&Container> = group
        .iter()
        .flat_map(|group| group.containers.iter())
        .filter(|c| !is_sandbox(c))
        .collect();
    let active = containers.iter().filter(|c| c.state.is_active()).count();
    let project = view
        .project
        .as_ref()
        .filter(|_| matches!(key, GroupKey::Project(_)));

    let doors = open_row::doors(&containers);
    let stats = workspace.stats();
    let selected = workspace.selected().map(|c| c.id.clone());
    let inspector =
        workspace.card_open() && containers.iter().any(|c| Some(&c.id) == selected.as_ref());
    let titles: Vec<String> = containers.iter().map(|c| service_name(c)).collect();
    let cards = containers.iter().map(|container| {
        let service = service_name(container);
        // A scaled service has several containers; their names tell them apart.
        let scaled = titles.iter().filter(|t| **t == service).count() > 1;
        let title = if scaled {
            container.display_name()
        } else {
            service.clone()
        };
        let card = Card {
            container,
            title,
            detail: view.details.get(&container.id).map(|(_, detail)| detail),
            history: stats.get(&container.id),
            exits: view.recent_exits(&service),
            selected: inspector && selected.as_ref() == Some(&container.id),
            pending: workspace.is_pending(&container.id),
        };
        service_card::render(card, &handle, &weak, &palette).into_any_element()
    });
    let mut grid: Vec<AnyElement> = cards.collect();
    if let Some(project) = project {
        let file = project
            .config_files
            .first()
            .and_then(|file| file.rsplit(['/', '\\']).next())
            .unwrap_or("the Compose file");
        grid.push(tasks_card::render(&view.tasks, file, &weak, &palette).into_any_element());
    }
    if containers.is_empty() {
        grid.insert(0, empty(&key, &palette).into_any_element());
    }

    div()
        .size_full()
        .flex()
        .flex_col()
        .child(header::render(
            &key,
            project,
            (active, containers.len()),
            &handle,
            workspace,
            &weak,
            &palette,
        ))
        .children((!doors.is_empty()).then(|| open_row::render(doors, &weak, &palette)))
        .child(
            div()
                .id("project-cards")
                .flex_shrink_0()
                .max_h(relative(0.6))
                .overflow_y_scroll()
                .px(px(28.))
                .child(
                    div()
                        .grid()
                        .grid_cols(if inspector { 2 } else { 3 })
                        .gap(px(12.))
                        .children(grid),
                ),
        )
        .child(view.log.clone())
        .into_any_element()
}

/// The card in place of the services when none is left, for example after Down.
fn empty(key: &GroupKey, palette: &Palette) -> Div {
    let text = match key {
        GroupKey::Project(name) => {
            format!("{name} has no containers. Up creates them from its Compose files.")
        }
        _ => "No containers are left here.".into(),
    };
    div()
        .p(px(14.))
        .rounded(px(14.))
        .border_1()
        .border_color(palette.sep)
        .bg(palette.card)
        .text_size(px(12.))
        .text_color(palette.text2)
        .child(text)
}
