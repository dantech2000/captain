use captain_core::store::ContainerStore;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use crate::theme::Palette;
use crate::workspace::{Page, active_workspace};

/// The Compose projects on the engine, with running counts. A click shows the
/// Containers page with only that project.
pub fn render(store: &ContainerStore, palette: &Palette) -> impl IntoElement {
    let projects = store.projects();
    div()
        .flex()
        .flex_col()
        .gap(px(2.))
        .when(!projects.is_empty(), |this| {
            this.child(
                div()
                    .px(px(10.))
                    .pt(px(18.))
                    .pb(px(6.))
                    .text_size(px(11.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(palette.text3)
                    .child("Projects"),
            )
        })
        .children(projects.into_iter().map(|group| {
            let name = group.project().unwrap_or_default().to_string();
            let color = palette.project_color(&name);
            let running = group.running_count();
            let state = if running == 0 {
                "stopped".to_string()
            } else {
                format!("{running}/{}", group.containers.len())
            };
            let hover = palette.nav_selected;
            let project = name.clone();
            div()
                .id(SharedString::from(format!("sidebar-project-{name}")))
                .h(px(30.))
                .px(px(10.))
                .flex()
                .items_center()
                .gap(px(10.))
                .rounded(px(8.))
                .cursor_pointer()
                .hover(move |style| style.bg(hover))
                .on_click(move |_, _, cx| {
                    // The sidebar gets only the store, so find the workspace here.
                    if let Some(workspace) = active_workspace(cx) {
                        workspace.update(cx, |workspace, cx| {
                            workspace.set_project_filter(project.clone(), cx);
                            workspace.set_page(Page::Containers, cx);
                        });
                    }
                })
                .child(project_badge(&name, color, px(16.), palette))
                .child(div().flex_1().truncate().child(name))
                .child(
                    div()
                        .text_size(px(11.))
                        .text_color(palette.text3)
                        .child(state),
                )
        }))
}

/// A rounded tile with the project's first letter in its color.
pub fn project_badge(name: &str, color: Hsla, size: Pixels, palette: &Palette) -> Div {
    let initial = name
        .chars()
        .next()
        .unwrap_or('·')
        .to_uppercase()
        .to_string();
    div()
        .size(size)
        .flex_shrink_0()
        .rounded(size * 0.3)
        .bg(palette.tint(color))
        .text_color(color)
        .text_size(size * 0.55)
        .font_weight(FontWeight::BOLD)
        .flex()
        .items_center()
        .justify_center()
        .child(initial)
}
