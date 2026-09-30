//! The question after Save and apply: what `up` would change, per service.

use captain_core::model::ComposeProject;
use captain_core::project_files::{ChangeKind, UpPreview};
use gpui_kit::component::WindowExt;
use gpui_kit::*;

use super::FileEditor;
use crate::project::ProjectView;
use crate::theme::Palette;
use crate::widgets::confirm_footer;

/// Shows `preview` and runs `up` when the user confirms.
pub fn open(
    preview: UpPreview,
    project: ComposeProject,
    view: Entity<ProjectView>,
    editor: Entity<FileEditor>,
    window: &mut Window,
    cx: &mut App,
) {
    window.open_alert_dialog(cx, move |alert, _, cx| {
        let palette = Palette::of(cx);
        let (view, project, editor) = (view.clone(), project.clone(), editor.clone());
        let title = if preview.is_empty() {
            "Nothing changes"
        } else {
            "Apply the saved files?"
        };
        alert
            .title(title)
            .description(summary(&preview, &palette))
            .footer(confirm_footer(
                "Apply",
                "Run docker compose up with the saved files. The project log shows its output.",
            ))
            .on_ok(move |_, _, cx| {
                let (project, editor) = (project.clone(), editor.clone());
                view.update(cx, |view, cx| {
                    view.apply_files(project, Vec::new(), editor, cx)
                });
                true
            })
    });
}

/// One row per container and image step, then Compose's warnings.
fn summary(preview: &UpPreview, palette: &Palette) -> Div {
    let rows = preview.changes.iter().map(|change| {
        let color = match change.change {
            ChangeKind::Recreate | ChangeKind::Create | ChangeKind::Start => palette.accent,
            ChangeKind::Remove => palette.red,
            ChangeKind::Unchanged | ChangeKind::Other(_) => palette.text3,
        };
        row(change.change.label(), &change.service, color, palette)
    });
    let images = preview.images.iter().map(|step| {
        let action = if step.build { "Build" } else { "Pull" };
        row(action, &step.name, palette.info, palette)
    });
    let intro = if preview.is_empty() {
        "The containers already match the saved files. Up would change nothing."
    } else {
        "Captain ran docker compose up as a dry run. These are the changes it would make:"
    };
    div()
        .flex()
        .flex_col()
        .gap(px(4.))
        .text_size(px(13.))
        .child(div().pb(px(4.)).text_color(palette.text2).child(intro))
        .children(images)
        .children(rows)
        .children(preview.warnings.iter().map(|warning| {
            div()
                .pt(px(4.))
                .text_size(px(12.))
                .text_color(palette.warn_text)
                .child(warning.clone())
        }))
}

fn row(action: &str, name: &str, color: Hsla, palette: &Palette) -> Div {
    div()
        .flex()
        .gap(px(10.))
        .child(
            div()
                .w(px(90.))
                .flex_shrink_0()
                .text_color(palette.readable(color))
                .font_weight(FontWeight::MEDIUM)
                .child(action.to_string()),
        )
        .child(div().text_color(palette.text).child(name.to_string()))
}
