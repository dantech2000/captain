use captain_core::store::ContainerFilter;
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::*;

use super::{column_header, empty_state, header, project_card, selection_bar, stat_tiles};
use crate::engine_host::{HostModel, connection_screen, host_model, host_screen};
use crate::theme::Palette;
use crate::workspace::Workspace;

/// The Containers page: stat tiles and containers grouped into project cards.
pub struct ContainersView {
    workspace: Entity<Workspace>,
    /// Captain Engine, whose screens replace the list while it is not running.
    host: Option<Entity<HostModel>>,
    _observe: Vec<Subscription>,
}

impl ContainersView {
    pub fn new(workspace: Entity<Workspace>, cx: &mut Context<Self>) -> Self {
        let host = host_model(cx);
        let mut observe = vec![cx.observe(&workspace, |_, _, cx| cx.notify())];
        observe.extend(
            host.as_ref()
                .map(|host| cx.observe(host, |_, _, cx| cx.notify())),
        );
        Self {
            workspace,
            host,
            _observe: observe,
        }
    }
}

impl Render for ContainersView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = Palette::of(cx);
        let workspace = self.workspace.read(cx);

        // While Captain Engine is not running, its screens replace the list.
        let screen = self
            .host
            .as_ref()
            .and_then(|host| host_screen(host, &palette, cx))
            .or_else(|| connection_screen(workspace, &palette, cx));
        let body = match screen {
            Some(screen) => screen,
            None if !workspace.is_loaded() => div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .text_color(palette.text2)
                .child("Connecting to the engine...")
                .into_any_element(),
            None => self.list(workspace, &palette).into_any_element(),
        };

        div()
            .size_full()
            .flex()
            .flex_col()
            .child(header::render(&self.workspace, workspace, &palette))
            .child(div().flex_1().min_h_0().child(body))
    }
}

impl ContainersView {
    fn list(&self, workspace: &Workspace, palette: &Palette) -> impl IntoElement {
        let groups = workspace.visible_groups();
        let projects = workspace.compose_projects();
        let filtered =
            workspace.filter() != ContainerFilter::All || workspace.project_filter().is_some();
        let content = if groups.is_empty() {
            empty_state::render(filtered, palette).into_any_element()
        } else {
            div()
                .flex()
                .flex_col()
                .gap(px(10.))
                .children(groups.into_iter().map(|group| {
                    let project = projects
                        .iter()
                        .find(|p| group.project() == Some(p.name.as_str()));
                    project_card::render(group, project, &self.workspace, workspace, palette)
                }))
                .into_any_element()
        };

        div()
            .size_full()
            .flex()
            .flex_col()
            .child(stat_tiles::render(workspace, palette))
            .children(
                workspace
                    .bulk()
                    .is_bulk()
                    .then(|| selection_bar::render(&self.workspace, workspace, palette)),
            )
            .child(column_header::render(palette))
            .child(
                div()
                    .id("container-list")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scrollbar()
                    .px(px(12.))
                    .pt(px(10.))
                    .pb(px(16.))
                    .child(content),
            )
    }
}
