use captain_core::model::count_label;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use super::ProjectView;
use crate::help::HelpExt;
use crate::theme::Palette;

/// What the Project page shows under its header.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ProjectTab {
    /// The Open row, the service cards, the tasks, and the log.
    #[default]
    Overview,
    /// Ports, services by network, and volumes, with staged changes.
    Map,
}

/// "Overview | Map", and a pill for the staged changes of the shown containers.
pub fn render(
    tab: ProjectTab,
    staged: usize,
    view: &WeakEntity<ProjectView>,
    palette: &Palette,
) -> Div {
    let shadow = if palette.dark { 0.4 } else { 0.12 };
    let segments = [
        (
            ProjectTab::Overview,
            "Overview",
            "Show the services as cards, with the tasks and one log for all of them.",
        ),
        (
            ProjectTab::Map,
            "Map",
            "Show the ports, networks, and volumes of the services, and stage changes to their limits.",
        ),
    ];
    let control = div()
        .flex()
        .p(px(2.))
        .rounded(px(9.))
        .bg(palette.field)
        .children(segments.into_iter().map(|(choice, label, help)| {
            let selected = choice == tab;
            let view = view.clone();
            div()
                .id(SharedString::from(format!("project-tab-{label}")))
                .h(px(26.))
                .px(px(12.))
                .flex()
                .items_center()
                .rounded(px(7.))
                .cursor_pointer()
                .text_size(px(12.))
                .font_weight(FontWeight::MEDIUM)
                .text_color(if selected {
                    palette.text
                } else {
                    palette.text2
                })
                .when(selected, |this| {
                    this.bg(palette.segment).shadow(vec![BoxShadow {
                        color: hsla(0., 0., 0., shadow),
                        offset: point(px(0.), px(1.)),
                        blur_radius: px(2.),
                        spread_radius: px(0.),
                        inset: false,
                    }])
                })
                .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                .on_click(move |_, _, cx| {
                    view.update(cx, |view, cx| view.show_tab(choice, cx)).ok();
                })
                .child(label)
                .help(help)
        }));
    let pill = (staged > 0).then(|| {
        let view = view.clone();
        let text = count_label(staged, "staged change");
        div()
            .id("project-staged-pill")
            .h(px(26.))
            .px(px(10.))
            .flex()
            .items_center()
            .gap(px(6.))
            .rounded(px(13.))
            .bg(palette.tint(palette.orange))
            .text_color(palette.warn_text)
            .text_size(px(12.))
            .font_weight(FontWeight::SEMIBOLD)
            .cursor_pointer()
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .on_click(move |_, _, cx| {
                view.update(cx, |view, cx| view.show_tab(ProjectTab::Map, cx))
                    .ok();
            })
            .child(div().size(px(7.)).rounded_full().bg(palette.orange))
            .child(text.clone())
            .help(format!(
                "Open the map to check or apply the {text}. Nothing changes until you apply."
            ))
    });
    div()
        .flex()
        .items_center()
        .gap(px(10.))
        .child(control)
        .children(pill)
}
