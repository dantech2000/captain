//! The problems from the checks under the editor. A click moves the cursor to
//! the problem's line.

use captain_core::project_files::{LineProblem, Severity};
use gpui_kit::component::input::Position;
use gpui_kit::*;

use super::FileEditor;
use crate::help::HelpExt;
use crate::theme::Palette;

pub fn render(
    editor: &FileEditor,
    this: &Entity<FileEditor>,
    palette: &Palette,
) -> Option<Stateful<Div>> {
    if editor.problems.is_empty() {
        return None;
    }
    let rows = editor
        .problems
        .iter()
        .enumerate()
        .map(|(ix, problem)| problem_row(ix, problem, this, palette));
    Some(
        div()
            .id("file-problems")
            .max_h(px(120.))
            .overflow_y_scroll()
            .flex()
            .flex_col()
            .gap(px(2.))
            .children(rows),
    )
}

fn problem_row(
    ix: usize,
    problem: &LineProblem,
    this: &Entity<FileEditor>,
    palette: &Palette,
) -> Stateful<Div> {
    let color = match problem.severity {
        Severity::Error => palette.red,
        Severity::Warning => palette.orange,
    };
    let place = problem
        .line
        .map_or_else(|| "File".to_string(), |line| format!("Line {}", line + 1));
    let (line, hover) = (problem.line, palette.hover);
    let this = this.clone();
    div()
        .id(("file-problem", ix))
        .flex()
        .items_center()
        .gap(px(8.))
        .px(px(6.))
        .py(px(3.))
        .rounded(px(6.))
        .text_size(px(12.))
        .cursor_pointer()
        .hover(move |style| style.bg(hover))
        .on_click(move |_, window, cx| {
            let Some(line) = line else {
                return;
            };
            let editor = this.read(cx).editor.clone();
            editor.update(cx, |state, cx| {
                state.set_cursor_position(Position::new(line as u32, 0), window, cx);
            });
        })
        .child(div().size(px(7.)).rounded_full().bg(color))
        .child(div().text_color(palette.text2).child(place))
        .child(
            div()
                .text_color(palette.text)
                .child(problem.message.clone()),
        )
        .help("Move the cursor to this line.")
}
