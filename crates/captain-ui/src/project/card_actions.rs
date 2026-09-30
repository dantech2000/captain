use captain_core::format::bytes_label;
use captain_core::model::{ContainerAction, ContainerState};
use gpui_kit::assets::IconName;
use gpui_kit::component::Icon;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use super::ProjectView;
use super::service_card::Card;
use crate::help::HelpExt;
use crate::icons::{CaptainIcon, cap_icon};
use crate::theme::Palette;
use crate::workspace::{InspectorTab, Workspace};

/// The smallest limit "Raise memory" offers.
const MIN_RAISED: u64 = 512 * 1024 * 1024;

/// The new limit after an out-of-memory kill: twice the old one, at least 512 MB.
pub fn raised_limit(limit: u64) -> u64 {
    limit.saturating_mul(2).max(MIN_RAISED)
}

/// Buttons for the state the card is in: Resume a paused container, Start a
/// stopped one, and raise the memory limit after an out-of-memory kill.
pub fn context(
    card: &Card,
    handle: &Entity<Workspace>,
    view: &WeakEntity<ProjectView>,
    palette: &Palette,
) -> Vec<Stateful<Div>> {
    let container = card.container;
    let name = &card.title;
    let mut buttons = Vec::new();
    let limit = card
        .detail
        .filter(|d| d.oom_killed)
        .map(|d| d.memory_limit)
        .filter(|limit| *limit > 0);
    if let Some(limit) = limit {
        let raised = raised_limit(limit);
        let label = bytes_label(raised);
        let (view, id, title) = (view.clone(), container.id.clone(), name.clone());
        buttons.push(
            button(
                format!("raise-{}", container.id),
                format!("Raise memory to {label}"),
                palette.red,
                !card.pending,
                palette,
                move |cx| {
                    view.update(cx, |view, cx| view.raise_memory(&id, &title, raised, cx))
                        .ok();
                },
            )
            .help(format!(
                "Set the memory limit of {name} to {label} now. Compose sets the old limit again when it recreates {name}."
            )),
        );
    }
    let action = match container.state {
        ContainerState::Paused => Some((
            ContainerAction::Unpause,
            "Resume",
            format!("Resume {name}. Its frozen processes go on where they stopped."),
        )),
        ContainerState::Exited | ContainerState::Created if limit.is_none() => Some((
            ContainerAction::Start,
            "Start",
            format!("Start {name} again."),
        )),
        _ => None,
    };
    if let Some((action, label, help)) = action {
        let (handle, id) = (handle.clone(), container.id.clone());
        let color = if action == ContainerAction::Unpause {
            palette.orange
        } else {
            palette.accent
        };
        buttons.push(
            button(
                format!("{label}-{}", container.id),
                label.to_string(),
                color,
                !card.pending,
                palette,
                move |cx| {
                    handle.update(cx, |workspace, cx| {
                        workspace.run_action(id.clone(), action, cx)
                    })
                },
            )
            .help(help),
        );
    }
    buttons
}

/// Logs and Shell: they open the inspector at that tab.
pub fn tools(card: &Card, handle: &Entity<Workspace>, palette: &Palette) -> [Stateful<Div>; 2] {
    let running = card.container.state == ContainerState::Running;
    let name = &card.title;
    let logs = tool(
        format!("card-logs-{}", card.container.id),
        Icon::new(IconName::FileText)
            .size(px(13.))
            .into_any_element(),
        true,
        card,
        handle,
        InspectorTab::Logs,
        palette,
    )
    .help(format!("Show the logs of {name} in the inspector."));
    let shell = tool(
        format!("card-shell-{}", card.container.id),
        cap_icon(CaptainIcon::Exec, px(14.), palette.text2).into_any_element(),
        running,
        card,
        handle,
        InspectorTab::Terminal,
        palette,
    )
    .help(if running {
        format!("Open a shell in {name}.")
    } else {
        format!("{name} does not run, so it has no shell.")
    });
    [logs, shell]
}

fn tool(
    id: String,
    icon: AnyElement,
    enabled: bool,
    card: &Card,
    handle: &Entity<Workspace>,
    tab: InspectorTab,
    palette: &Palette,
) -> Stateful<Div> {
    let (handle, container) = (handle.clone(), card.container.id.clone());
    let hover = palette.nav_selected;
    div()
        .id(SharedString::from(id))
        .size(px(26.))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(7.))
        .bg(palette.button)
        .text_color(palette.text2)
        .when(!enabled, |this| this.opacity(0.4))
        .when(enabled, |this| {
            this.cursor_pointer()
                .hover(move |style| style.bg(hover))
                .on_click(move |_, _, cx| {
                    cx.stop_propagation();
                    handle.update(cx, |workspace, cx| {
                        workspace.open_card_tab(container.clone(), tab, cx)
                    });
                })
        })
        .child(icon)
}

fn button(
    id: String,
    label: String,
    color: Hsla,
    enabled: bool,
    palette: &Palette,
    on_click: impl Fn(&mut App) + 'static,
) -> Stateful<Div> {
    let hover = color.alpha(if palette.dark { 0.3 } else { 0.2 });
    div()
        .id(SharedString::from(id))
        .h(px(26.))
        .px(px(10.))
        .flex()
        .items_center()
        .rounded(px(7.))
        .bg(palette.tint(color))
        .text_color(palette.readable(color))
        .text_size(px(11.))
        .font_weight(FontWeight::SEMIBOLD)
        .when(!enabled, |this| this.opacity(0.5))
        .when(enabled, |this| {
            this.cursor_pointer()
                .hover(move |style| style.bg(hover))
                .on_click(move |_, _, cx| {
                    cx.stop_propagation();
                    on_click(cx);
                })
        })
        .child(label)
}
