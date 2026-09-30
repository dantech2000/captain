//! The right side of the Files tab: the toolbar, the reload bar, the editor, and
//! the list of problems from the checks.

use captain_core::project_files::{FileKind, Severity};
use gpui_kit::component::input::Editor;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use super::file_editor::DiskState;
use super::{FileEditor, problem_list};
use crate::help::HelpExt;
use crate::theme::Palette;
use crate::widgets::{ButtonTone, inline_error, text_button};

pub fn render(editor: &FileEditor, cx: &mut Context<FileEditor>) -> Div {
    let palette = Palette::of(cx);
    let this = cx.entity();
    div()
        .size_full()
        .min_w_0()
        .flex()
        .flex_col()
        .gap(px(8.))
        .child(toolbar(editor, &this, &palette))
        .children(
            editor
                .error
                .clone()
                .map(|error| inline_error(error, &palette)),
        )
        .children(
            editor
                .action_error
                .clone()
                .map(|error| inline_error(error, &palette)),
        )
        .children(reload_bar(editor, &this, &palette))
        .child(
            div()
                .flex_1()
                .min_h_0()
                .child(Editor::new(&editor.editor).size_full()),
        )
        .children(problem_list::render(editor, &this, &palette))
}

fn toolbar(editor: &FileEditor, this: &Entity<FileEditor>, palette: &Palette) -> Div {
    let busy = editor.busy.clone();
    let can_save = editor.dirty && editor.disk != DiskState::Missing && busy.is_none();
    let save = {
        let this = this.clone();
        text_button(
            "file-save",
            "Save",
            ButtonTone::Accent,
            can_save,
            palette,
            move |_, _, cx| {
                this.update(cx, |this, cx| this.save(cx));
            },
        )
        .help(format!(
            "Write your changes to {}. Captain never overwrites a change made outside.",
            editor.label()
        ))
    };
    let actions: Vec<AnyElement> = match editor.file.kind {
        FileKind::Compose => vec![apply_button(editor, this, busy.is_none(), palette)],
        FileKind::Dockerfile => {
            let errors = editor
                .problems
                .iter()
                .any(|p| p.severity == Severity::Error);
            editor
                .file
                .services
                .iter()
                .map(|service| {
                    rebuild_button(service, editor, this, busy.is_none(), errors, palette)
                })
                .collect()
        }
    };
    div()
        .flex()
        .items_center()
        .gap(px(8.))
        .child(
            div()
                .text_size(px(13.))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(palette.text)
                .child(editor.label()),
        )
        .when(editor.dirty, |row| {
            row.child(
                div()
                    .text_size(px(11.))
                    .text_color(palette.warn_text)
                    .child("Unsaved"),
            )
        })
        .child(
            div()
                .flex_1()
                .min_w_0()
                .truncate()
                .text_size(px(11.))
                .text_color(palette.text2)
                .child(busy.unwrap_or_else(|| status(editor).into())),
        )
        .child(save)
        .children(actions)
}

/// "Checking...", "No problems", or "2 errors, 1 warning".
fn status(editor: &FileEditor) -> String {
    if editor.runner.is_none() {
        return "Checks need docker compose.".into();
    }
    if editor.checking {
        return "Checking...".into();
    }
    if let Some(error) = &editor.check_error {
        return format!("The check did not run: {error}");
    }
    let count = |severity| {
        editor
            .problems
            .iter()
            .filter(|p| p.severity == severity)
            .count()
    };
    match (count(Severity::Error), count(Severity::Warning)) {
        (0, 0) => "No problems".into(),
        (errors, warnings) => [(errors, "error"), (warnings, "warning")]
            .into_iter()
            .filter(|(n, _)| *n > 0)
            .map(|(n, word)| captain_core::model::count_label(n, word))
            .collect::<Vec<_>>()
            .join(", "),
    }
}

fn apply_button(
    editor: &FileEditor,
    this: &Entity<FileEditor>,
    enabled: bool,
    palette: &Palette,
) -> AnyElement {
    let (this, view) = (this.clone(), editor.view.clone());
    text_button(
        "file-save-apply",
        "Save and apply",
        ButtonTone::Accent,
        enabled,
        palette,
        move |_, window, cx| {
            let this = this.clone();
            view.update(cx, |view, cx| view.save_and_apply(this, window, cx))
                .ok();
        },
    )
    .help(
        "Save, then preview which services up recreates, creates, or removes. Nothing runs \
         until you confirm.",
    )
    .into_any_element()
}

/// Waits while the build check finds errors: a build with them fails.
fn rebuild_button(
    service: &str,
    editor: &FileEditor,
    this: &Entity<FileEditor>,
    idle: bool,
    errors: bool,
    palette: &Palette,
) -> AnyElement {
    let (this, name, view) = (this.clone(), service.to_string(), editor.view.clone());
    text_button(
        SharedString::from(format!("file-rebuild-{service}")),
        format!("Rebuild {service}"),
        ButtonTone::Accent,
        idle && !errors,
        palette,
        move |_, _, cx| {
            let this = this.clone();
            let name = name.clone();
            view.update(cx, |view, cx| view.save_and_rebuild(this, name, cx))
                .ok();
        },
    )
    .help(if errors {
        format!("Fix the errors the build check found first. A build of {service} with them fails.")
    } else {
        format!(
            "Save the Dockerfile, then build the image of {service} again and recreate its \
             container."
        )
    })
    .into_any_element()
}

fn reload_bar(editor: &FileEditor, this: &Entity<FileEditor>, palette: &Palette) -> Option<Div> {
    let text = match editor.disk {
        DiskState::Same => return None,
        DiskState::Changed => "This file changed on disk. Your edits here are not saved.",
        DiskState::Missing => "Captain cannot read this file. It may have moved or been deleted.",
    };
    let reload = {
        let this = this.clone();
        text_button(
            "file-reload",
            "Reload",
            ButtonTone::Accent,
            true,
            palette,
            move |_, window, cx| {
                this.update(cx, |this, cx| this.load(window, cx));
            },
        )
        .help("Read the file from disk again. Your unsaved edits here are dropped.")
    };
    let keep = (editor.disk == DiskState::Changed).then(|| {
        let this = this.clone();
        text_button(
            "file-keep-mine",
            "Keep mine",
            ButtonTone::Danger,
            true,
            palette,
            move |_, _, cx| {
                this.update(cx, |this, cx| this.keep_mine(cx));
            },
        )
        .help("Keep your text. The next Save replaces the version on disk.")
    });
    Some(
        div()
            .flex()
            .items_center()
            .gap(px(8.))
            .px(px(10.))
            .py(px(6.))
            .rounded(px(8.))
            .bg(palette.tint(palette.orange))
            .text_size(px(12.))
            .text_color(palette.warn_text)
            .child(div().flex_1().child(text))
            .child(reload)
            .children(keep),
    )
}
