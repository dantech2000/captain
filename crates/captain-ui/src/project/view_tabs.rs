use captain_core::model::count_label;
use gpui_kit::*;

use super::ProjectView;
use crate::help::HelpExt;
use crate::theme::Palette;
use crate::widgets::{Segment, segmented};

/// What the Project page shows under its header.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ProjectTab {
    /// The Open row, the service cards, the tasks, and the log.
    #[default]
    Overview,
    /// Ports, services by network, and volumes, with staged changes.
    Map,
    /// The Compose files and Dockerfiles in an editor. Compose projects only.
    Files,
}

/// "Overview | Map | Files", and a pill for the staged changes of the shown
/// containers. `files` is false for an entry that is not a Compose project.
pub fn render(
    tab: ProjectTab,
    staged: usize,
    files: bool,
    view: &WeakEntity<ProjectView>,
    palette: &Palette,
) -> Div {
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
        (
            ProjectTab::Files,
            "Files",
            "Edit the Compose files and Dockerfiles of this project, with checks as you type.",
        ),
    ];
    let segments = segments
        .into_iter()
        .filter(|(choice, _, _)| files || *choice != ProjectTab::Files)
        .map(|(choice, label, help)| {
            let view = view.clone();
            Segment {
                label: label.into(),
                selected: choice == tab,
                help: help.into(),
                on_click: Box::new(move |_, cx| {
                    view.update(cx, |view, cx| view.show_tab(choice, cx)).ok();
                }),
            }
        })
        .collect();
    // The header moves the window; a click on a segment must not.
    let control = segmented("project-tab", segments, palette)
        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation());
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
