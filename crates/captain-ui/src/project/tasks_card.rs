use captain_core::model::ProjectTask;
use captain_core::store::ProjectRun;
use gpui_kit::assets::IconName;
use gpui_kit::component::spinner::Spinner;
use gpui_kit::component::{Icon, Sizable};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use super::ProjectView;
use super::tasks::{TaskList, TaskState};
use crate::help::HelpExt;
use crate::theme::Palette;

/// How many lines of a task's output the card shows.
const OUTPUT_LINES: usize = 6;

/// The guide section on `x-captain.tasks`.
const TASKS_GUIDE: &str = concat!(
    env!("CARGO_PKG_REPOSITORY"),
    "/blob/main/docs/guide/projects-and-tasks.md#tasks"
);

/// The Tasks card: a button per task in `x-captain.tasks`, the end of the last run,
/// or a line with a link to the guide on how to add tasks. `project` is the Compose
/// project's name, and `file` its Compose file's name.
pub fn render(
    state: &TaskState,
    project: &str,
    file: &str,
    view: &WeakEntity<ProjectView>,
    palette: &Palette,
) -> Div {
    let card = div()
        .min_w_0()
        .flex()
        .flex_col()
        .gap(px(8.))
        .p(px(14.))
        .rounded(px(14.))
        .border_1()
        .border_dashed()
        .border_color(palette.border_strong)
        .child(div().font_weight(FontWeight::BOLD).child("Tasks"));
    let run = state.runs.get(project);
    match &state.list {
        TaskList::None => card.child(note("Tasks need Docker Compose.", palette)),
        TaskList::Loading => card.child(note(format!("Reading {file}..."), palette)),
        TaskList::Failed(error) => {
            card.child(note(format!("Cannot read the tasks: {error}"), palette))
        }
        TaskList::Ready(tasks) if tasks.tasks.is_empty() && tasks.problems.is_empty() => {
            card.child(no_tasks(file, palette))
        }
        TaskList::Ready(tasks) => card
            .child(note(format!("From x-captain.tasks in {file}"), palette))
            .child(
                div().flex().flex_wrap().gap(px(6.)).children(
                    tasks
                        .tasks
                        .iter()
                        .map(|task| task_button(task, run, view, palette)),
                ),
            )
            .children(tasks.problems.iter().map(|problem| {
                div()
                    .text_size(px(11.))
                    .text_color(palette.warn_text)
                    .child(problem.clone())
            }))
            .children(run.and_then(|run| last_run(run, palette))),
    }
}

fn note(text: impl Into<SharedString>, palette: &Palette) -> Div {
    div()
        .text_size(px(11.))
        .text_color(palette.text3)
        .child(text.into())
}

/// One line: no tasks yet, and a link to the guide section on tasks.
fn no_tasks(file: &str, palette: &Palette) -> Div {
    note(format!("No x-captain.tasks in {file}."), palette)
        .flex()
        .gap(px(6.))
        .child(
            div()
                .id("tasks-guide")
                .text_color(palette.link)
                .cursor_pointer()
                .on_click(|_, _, cx| cx.open_url(TASKS_GUIDE))
                .child("How to add tasks")
                .help("Open the guide section on tasks: named commands in the Compose file."),
        )
}

fn task_button(
    task: &ProjectTask,
    run: Option<&ProjectRun>,
    view: &WeakEntity<ProjectView>,
    palette: &Palette,
) -> Stateful<Div> {
    let running_task = run
        .and_then(|run| run.running.as_ref())
        .map(|(_, name)| name);
    let running = running_task == Some(&task.name);
    let idle = running_task.is_none();
    let hover = palette.nav_selected;
    let (view, run) = (view.clone(), task.clone());
    div()
        .id(SharedString::from(format!("task-{}", task.name)))
        .h(px(28.))
        .px(px(10.))
        .flex()
        .items_center()
        .gap(px(6.))
        .rounded(px(8.))
        .border_1()
        .border_color(palette.border_strong)
        .bg(palette.field)
        .text_size(px(12.))
        .when(!idle && !running, |this| this.opacity(0.5))
        .when(idle, |this| {
            this.cursor_pointer()
                .hover(move |style| style.bg(hover))
                .on_click(move |_, _, cx| {
                    let task = run.clone();
                    view.update(cx, |view, cx| view.run_task(task, cx)).ok();
                })
        })
        .child(if running {
            Spinner::new()
                .xsmall()
                .color(palette.text2)
                .into_any_element()
        } else {
            Icon::new(IconName::Play)
                .size(px(10.))
                .text_color(palette.green)
                .into_any_element()
        })
        .child(task.name.clone())
        .child(
            div()
                .text_size(px(10.5))
                .text_color(palette.text3)
                .child(task.service.clone()),
        )
        .help(format!(
            "Run `{}` in {} (from x-captain.tasks).",
            task.command.display(),
            task.service
        ))
}

/// The exit code and the last lines of the last run.
fn last_run(run: &ProjectRun, palette: &Palette) -> Option<Div> {
    let (name, output) = run.last.as_ref()?;
    let lines: Vec<&str> = output.output.lines().collect();
    let tail = &lines[lines.len().saturating_sub(OUTPUT_LINES)..];
    let color = if output.exit_code == 0 {
        palette.green
    } else {
        palette.red
    };
    Some(
        div()
            .flex()
            .flex_col()
            .gap(px(4.))
            .child(
                div()
                    .text_size(px(11.))
                    .text_color(palette.readable(color))
                    .child(format!("{name} exited with {}", output.exit_code)),
            )
            .when(!tail.is_empty(), |this| {
                this.child(
                    div()
                        .p(px(8.))
                        .rounded(px(8.))
                        .bg(palette.terminal)
                        .font_family(palette.mono())
                        .text_size(px(10.5))
                        .text_color(palette.text2)
                        .children(
                            tail.iter()
                                .map(|line| div().truncate().child(line.to_string())),
                        ),
                )
            }),
    )
}
