use captain_core::store::ContainerFilter;
use gpui_kit::*;

use super::{column_header, empty_state, error_state, header, project_card, stat_tiles};
use crate::theme::Palette;
use crate::workspace::{Connection, Workspace};

/// The Containers page: stat tiles and containers grouped into project cards.
pub struct ContainersView {
    workspace: Entity<Workspace>,
    _observe: Subscription,
}

impl ContainersView {
    pub fn new(workspace: Entity<Workspace>, cx: &mut Context<Self>) -> Self {
        let observe = cx.observe(&workspace, |_, _, cx| cx.notify());
        Self {
            workspace,
            _observe: observe,
        }
    }
}

impl Render for ContainersView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = Palette::of(cx);
        let workspace = self.workspace.read(cx);

        let body = match workspace.connection() {
            Connection::Failed(error) => error_state::render(error, &palette).into_any_element(),
            _ if !workspace.is_loaded() => div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .text_color(palette.text2)
                .child("Connecting to the engine...")
                .into_any_element(),
            _ => self.list(workspace, &palette).into_any_element(),
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
                        .find(|p| group.project.as_ref() == Some(&p.name));
                    project_card::render(group, project, &self.workspace, workspace, palette)
                }))
                .into_any_element()
        };

        div()
            .size_full()
            .flex()
            .flex_col()
            .child(stat_tiles::render(workspace, palette))
            .child(column_header::render(palette))
            .child(
                div()
                    .id("container-list")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .px(px(12.))
                    .pt(px(10.))
                    .pb(px(16.))
                    .child(content),
            )
    }
}
