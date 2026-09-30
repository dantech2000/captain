use captain_core::format::{bytes_label, percent_label};
use captain_core::model::{Container, ContainerDetail, ContainerState};
use captain_core::project_map::{Rect, Setting, raised_memory};
use captain_core::store::StatsHistory;
use gpui_kit::assets::IconName;
use gpui_kit::component::Icon;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use super::scale::Scale;
use crate::help::HelpExt;
use crate::icons::{CaptainIcon, cap_icon};
use crate::project::ProjectView;
use crate::project::card_note::{card_note, state_help};
use crate::theme::Palette;
use crate::workspace::{InspectorTab, Workspace};

/// What one service node shows.
pub struct Node<'a> {
    pub container: &'a Container,
    pub title: &'a str,
    pub detail: Option<&'a ContainerDetail>,
    pub history: Option<&'a StatsHistory>,
    pub exits: usize,
    pub staged: bool,
    pub selected: bool,
    pub rect: Rect,
}

/// A service on the map: glyph, name, state, image, a note, and after an
/// out-of-memory kill, "Stage 512 MB" and Logs. A click opens the inspector.
pub fn render(
    node: Node,
    z: Scale,
    handle: &Entity<Workspace>,
    view: &WeakEntity<ProjectView>,
    palette: &Palette,
) -> Div {
    let container = node.container;
    let color = palette.container_state(container.state);
    let oom_limit = node
        .detail
        .filter(|d| d.oom_killed && container.state != ContainerState::Running)
        .map(|d| d.memory_limit)
        .filter(|limit| *limit > 0);
    let failing = oom_limit.is_some()
        || matches!(
            container.state,
            ContainerState::Restarting | ContainerState::Dead
        );
    let (note, note_color) = note(&node, palette);
    let border = if node.selected {
        palette.accent
    } else if failing {
        palette.red.alpha(0.6)
    } else {
        palette.border_strong
    };
    let (id, click) = (container.id.clone(), handle.clone());
    let label = container.state.label();
    let label = format!("{}{}", label[..1].to_uppercase(), &label[1..]);
    let body = div()
        .id(SharedString::from(format!("map-node-{}", container.id)))
        .size_full()
        .flex()
        .flex_col()
        .gap(z.px(6.))
        .px(z.px(12.))
        .py(z.px(11.))
        .rounded(z.px(13.))
        .bg(if failing {
            palette.card.blend(palette.red.alpha(0.08))
        } else {
            palette.card
        })
        .border_1()
        .border_color(border)
        .shadow_md()
        .cursor_pointer()
        .on_click(move |_, _, cx| {
            click.update(cx, |workspace, cx| workspace.toggle_card(id.clone(), cx))
        })
        .child(
            div()
                .flex()
                .items_center()
                .gap(z.px(8.))
                .child(cap_icon(CaptainIcon::Container, z.px(16.), color))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .truncate()
                        .text_size(z.px(13.))
                        .font_weight(FontWeight::BOLD)
                        .child(node.title.to_string()),
                )
                .child(
                    div()
                        .text_size(z.px(10.5))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(palette.readable(color))
                        .child(label),
                )
                .children(edit_button(&node, z, view, palette)),
        )
        .child(
            div()
                .text_size(z.px(11.))
                .font_family(palette.mono())
                .text_color(palette.text3)
                .truncate()
                .child(container.image.clone()),
        )
        .child(
            div()
                .text_size(z.px(11.))
                .text_color(note_color)
                .truncate()
                .child(note),
        )
        .when_some(oom_limit, |this, limit| {
            this.child(fix_row(&node, limit, z, handle, view, palette))
        })
        .help(format!(
            "{} Click to show {} in the inspector; click again to hide it.",
            state_help(container, node.detail, node.title),
            node.title
        ));
    z.place(node.rect)
        .child(body)
        .when(node.staged, |this| this.child(staged_tag(z, palette)))
}

/// CPU and memory while it runs, else the card's note: the exit reason or the state.
fn note(node: &Node, palette: &Palette) -> (String, Hsla) {
    let running = node.container.state == ContainerState::Running;
    match node.history.and_then(|h| h.latest()).filter(|_| running) {
        Some(sample) => (
            format!(
                "{} CPU · {}",
                percent_label(sample.cpu_percent),
                bytes_label(sample.memory_bytes)
            ),
            palette.text2,
        ),
        None => card_note(node.container, node.detail, node.exits, palette),
    }
}

/// "Stage 512 MB" and Logs, after an out-of-memory kill.
fn fix_row(
    node: &Node,
    limit: u64,
    z: Scale,
    handle: &Entity<Workspace>,
    view: &WeakEntity<ProjectView>,
    palette: &Palette,
) -> Div {
    let raised = raised_memory(limit);
    let label = bytes_label(raised);
    let id = node.container.id.clone();
    let (stage_view, stage_id, title) = (view.clone(), id.clone(), node.title.to_string());
    let stage = small_button(
        format!("map-stage-{id}"),
        format!("Stage {label}"),
        palette.tint(palette.red),
        palette.readable(palette.red),
        z,
    )
    .on_click(move |_, _, cx| {
        cx.stop_propagation();
        stage_view
            .update(cx, |view, cx| {
                view.stage(&stage_id, &title, Setting::Memory(raised), cx)
            })
            .ok();
    })
    .help(format!(
        "Stage a memory limit of {label} for {}, up from {}. Nothing changes until you apply.",
        node.title,
        bytes_label(limit)
    ));
    let logs_handle = handle.clone();
    let logs_id = id.clone();
    let logs = small_button(
        format!("map-logs-{id}"),
        "Logs".into(),
        palette.button,
        palette.text,
        z,
    )
    .on_click(move |_, _, cx| {
        cx.stop_propagation();
        logs_handle.update(cx, |workspace, cx| {
            workspace.open_card_tab(logs_id.clone(), InspectorTab::Logs, cx)
        });
    })
    .help(format!("Show the logs of {} in the inspector.", node.title));
    div().flex().gap(z.px(6.)).child(stage).child(logs)
}

fn small_button(id: String, label: String, bg: Hsla, fg: Hsla, z: Scale) -> Stateful<Div> {
    div()
        .id(SharedString::from(id))
        .h(z.px(22.))
        .px(z.px(8.))
        .flex()
        .items_center()
        .rounded(z.px(6.))
        .bg(bg)
        .text_color(fg)
        .text_size(z.px(10.5))
        .font_weight(FontWeight::SEMIBOLD)
        .cursor_pointer()
        .hover(|style| style.opacity(0.85))
        .child(label)
}

/// The pencil that opens the editor. It needs the `inspect` result for the values.
fn edit_button(
    node: &Node,
    z: Scale,
    view: &WeakEntity<ProjectView>,
    palette: &Palette,
) -> Option<Stateful<Div>> {
    node.detail?;
    let (view, id, title) = (
        view.clone(),
        node.container.id.clone(),
        node.title.to_string(),
    );
    let hover = palette.nav_selected;
    Some(
        div()
            .id(SharedString::from(format!("map-edit-{id}")))
            .size(z.px(20.))
            .flex()
            .items_center()
            .justify_center()
            .rounded(z.px(5.))
            .text_color(palette.text2)
            .cursor_pointer()
            .hover(move |style| style.bg(hover))
            .on_click(move |_, _, cx| {
                cx.stop_propagation();
                view.update(cx, |view, cx| view.edit(&id, &title, cx)).ok();
            })
            .child(Icon::new(IconName::Pencil).size(z.px(12.)))
            .help(format!(
                "Stage a new memory limit, CPU limit, or restart policy for {}.",
                node.title
            )),
    )
}

/// The "staged" tag on the top edge of a node with staged changes.
fn staged_tag(z: Scale, palette: &Palette) -> Div {
    div()
        .absolute()
        .top(z.px(-9.))
        .right(z.px(10.))
        .px(z.px(7.))
        .py(z.px(1.))
        .rounded(z.px(8.))
        .bg(palette.orange)
        .text_color(if palette.dark {
            palette.bg
        } else {
            palette.text
        })
        .text_size(z.px(10.))
        .font_weight(FontWeight::BOLD)
        .child("staged")
}
