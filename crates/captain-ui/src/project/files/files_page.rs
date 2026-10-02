use captain_core::project_files::{EditableFile, FileKind};
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use super::files_state::DockerfileList;
use crate::help::HelpExt;
use crate::icons::CaptainIcon;
use crate::project::ProjectView;
use crate::theme::Palette;
use crate::widgets::empty_note;

/// The file list on the left, the open file on the right.
pub fn render(
    view: &ProjectView,
    weak: &WeakEntity<ProjectView>,
    palette: &Palette,
    cx: &App,
) -> Div {
    let Some(project) = &view.project else {
        return div();
    };
    let dir = project.working_dir.clone().unwrap_or_default();
    let files = view.editable_files();
    let shown = view.shown_editor();
    let shown_path = shown.as_ref().map(|e| e.read(cx).file.path.clone());
    let rows = files.iter().enumerate().map(|(ix, file)| {
        let dirty = view
            .files
            .editors
            .get(&file.path)
            .is_some_and(|editor| editor.read(cx).dirty);
        let selected = shown_path.as_ref() == Some(&file.path);
        file_row(ix, file, &dir, dirty, selected, weak, palette)
    });
    let docker_note = match &view.files.dockerfiles {
        DockerfileList::Loading => Some("Reading the build contexts...".to_string()),
        DockerfileList::Failed(error) => Some(format!("No Dockerfiles: {error}")),
        _ => None,
    };
    let list = div()
        .id("project-files-list")
        .w(px(230.))
        .flex_shrink_0()
        .flex()
        .flex_col()
        .gap(px(2.))
        .overflow_y_scrollbar()
        .child(section_label("Files", palette))
        .children(rows)
        .children(docker_note.map(|note| {
            div()
                .px(px(8.))
                .pt(px(6.))
                .text_size(px(11.))
                .text_color(palette.text3)
                .child(note)
        }));
    let body = match shown {
        Some(editor) => div().flex_1().min_w_0().h_full().child(editor),
        None if files.is_empty() => div().flex_1().child(empty_note(
            CaptainIcon::Stack,
            "No files to edit",
            format!(
                "Captain edits the Compose files of {} that lie in the folder Compose ran in.",
                project.name
            ),
            palette,
        )),
        None => div().flex_1().child(empty_note(
            CaptainIcon::Stack,
            "Pick a file",
            "Choose a Compose file or Dockerfile on the left to edit it.",
            palette,
        )),
    };
    div()
        .flex_1()
        .min_h_0()
        .flex()
        .gap(px(16.))
        .px(px(28.))
        .pb(px(16.))
        .child(list)
        .child(body)
}

fn section_label(text: &'static str, palette: &Palette) -> Div {
    div()
        .px(px(8.))
        .pb(px(4.))
        .text_size(px(11.))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(palette.text3)
        .child(text)
}

fn file_row(
    ix: usize,
    file: &EditableFile,
    dir: &str,
    dirty: bool,
    selected: bool,
    weak: &WeakEntity<ProjectView>,
    palette: &Palette,
) -> Stateful<Div> {
    let label = file.label(std::path::Path::new(dir));
    let detail = match file.kind {
        FileKind::Compose => "Compose file".to_string(),
        FileKind::Dockerfile => format!("Builds {}", file.services.join(", ")),
    };
    let help = match file.kind {
        FileKind::Compose => format!("Edit {label}, a Compose file of this project."),
        FileKind::Dockerfile => format!(
            "Edit {label}, the Dockerfile of {}.",
            file.services.join(", ")
        ),
    };
    let (weak, file, hover) = (weak.clone(), file.clone(), palette.hover);
    div()
        .id(("project-file", ix))
        .px(px(8.))
        .py(px(6.))
        .rounded(px(8.))
        .cursor_pointer()
        .when(selected, |row| row.bg(palette.nav_selected))
        .when(!selected, |row| row.hover(move |style| style.bg(hover)))
        .on_click(move |_, window, cx| {
            let file = file.clone();
            weak.update(cx, |view, cx| view.open_file(file, window, cx))
                .ok();
        })
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(6.))
                .text_size(px(12.))
                .font_weight(FontWeight::MEDIUM)
                .text_color(palette.text)
                .child(div().min_w_0().truncate().child(label))
                .when(dirty, |row| {
                    row.child(div().size(px(6.)).rounded_full().bg(palette.orange))
                }),
        )
        .child(
            div()
                .text_size(px(11.))
                .text_color(palette.text2)
                .truncate()
                .child(detail),
        )
        .help(help)
}
