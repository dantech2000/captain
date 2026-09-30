use captain_core::format::{bytes_label, percent_label};
use captain_core::model::{Container, ContainerDetail, ContainerState, PortLink};
use captain_core::store::StatsHistory;
use gpui_kit::*;

use super::ProjectView;
use super::card_actions;
use super::card_note::{card_note, state_help};
use super::open_row::{door_help, follow};
use crate::help::HelpExt;
use crate::icons::{CaptainIcon, cap_icon};
use crate::theme::Palette;
use crate::widgets::{pill, scales, sparkline};
use crate::workspace::Workspace;

/// What one service card shows.
pub struct Card<'a> {
    pub container: &'a Container,
    /// The service, or the container name when it has none.
    pub title: String,
    pub detail: Option<&'a ContainerDetail>,
    /// The limit the last run ran out of, while a raise can still help.
    pub oom_limit: Option<u64>,
    pub history: Option<&'a StatsHistory>,
    /// Exits in the last two minutes.
    pub exits: usize,
    pub selected: bool,
    pub pending: bool,
}

/// A card: state glyph, name, image, state pill, a note, CPU and memory, the first
/// port, and actions. A click opens the inspector; a second click closes it.
pub fn render(
    card: Card,
    handle: &Entity<Workspace>,
    view: &WeakEntity<ProjectView>,
    palette: &Palette,
) -> Stateful<Div> {
    let container = card.container;
    let color = palette.container_state(container.state);
    let crashing = matches!(
        container.state,
        ContainerState::Restarting | ContainerState::Dead
    );
    let (note, note_color) = card_note(container, card.detail, card.exits, palette);
    let id = container.id.clone();
    let click = handle.clone();
    let border = if card.selected {
        palette.accent
    } else if crashing {
        palette.red.alpha(0.45)
    } else {
        palette.sep
    };
    div()
        .id(SharedString::from(format!("card-{}", container.id)))
        .min_w_0()
        .flex()
        .flex_col()
        .gap(px(10.))
        .p(px(14.))
        .rounded(px(14.))
        .bg(if crashing {
            palette.card.blend(palette.red.alpha(0.06))
        } else {
            palette.card
        })
        .border_1()
        .border_color(border)
        .cursor_pointer()
        .on_click(move |_, _, cx| {
            click.update(cx, |workspace, cx| workspace.toggle_card(id.clone(), cx))
        })
        .child(title_row(&card, color, palette))
        .child(
            div()
                .text_size(px(12.))
                .text_color(note_color)
                .truncate()
                .child(note),
        )
        .child(usage_row(&card, palette))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(6.))
                .children(card_actions::context(&card, handle, view, palette))
                .children(port_link(container, &card.title, view, palette))
                .child(div().flex_1())
                .children(card_actions::tools(&card, handle, palette)),
        )
        .help(format!(
            "Show the details of {} in the inspector. Click again to hide them.",
            card.title
        ))
}

fn title_row(card: &Card, color: Hsla, palette: &Palette) -> Div {
    let container = card.container;
    let label = container.state.label();
    let label = format!("{}{}", label[..1].to_uppercase(), &label[1..]);
    div()
        .flex()
        .items_center()
        .gap(px(10.))
        .child(cap_icon(CaptainIcon::Container, px(18.), color))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .gap(px(2.))
                .child(
                    div()
                        .text_size(px(14.))
                        .font_weight(FontWeight::BOLD)
                        .truncate()
                        .child(card.title.clone()),
                )
                .child(
                    div()
                        .text_size(px(11.))
                        .font_family(palette.mono())
                        .text_color(palette.text3)
                        .truncate()
                        .child(container.image.clone()),
                ),
        )
        .child(
            div()
                .id(SharedString::from(format!("card-state-{}", container.id)))
                .child(pill(label, palette.readable(color), palette.tint(color)))
                .help(state_help(container, card.detail, &card.title)),
        )
}

/// CPU, a CPU sparkline, and memory. A stopped container shows its memory limit.
fn usage_row(card: &Card, palette: &Palette) -> Div {
    let running = card.container.state == ContainerState::Running;
    let latest = card.history.and_then(|h| h.latest()).filter(|_| running);
    let cpu = latest.map_or("—".into(), |s| percent_label(s.cpu_percent));
    let limit = card.detail.map(|d| d.memory_limit).filter(|l| *l > 0);
    let memory = match (latest, limit) {
        (Some(sample), _) => bytes_label(sample.memory_bytes),
        (None, Some(limit)) => format!("{} limit", bytes_label(limit)),
        (None, None) => "—".into(),
    };
    let series = card.history.map(|h| h.cpu_series()).unwrap_or_default();
    let line = if running {
        palette.accent_fg
    } else {
        palette.gray
    };
    div()
        .flex()
        .items_center()
        .gap(px(10.))
        .text_size(px(11.))
        .text_color(palette.text2)
        .child(div().w(px(44.)).flex_shrink_0().child(cpu))
        .child(
            div().flex_1().min_w_0().h(px(22.)).child(
                sparkline(
                    series,
                    scales::CPU,
                    line,
                    None,
                    card.history.and_then(|h| h.last_at()),
                )
                .size_full(),
            ),
        )
        .child(div().flex_shrink_0().child(memory))
}

fn port_link(
    container: &Container,
    title: &str,
    view: &WeakEntity<ProjectView>,
    palette: &Palette,
) -> Option<Stateful<Div>> {
    let port = container.ports.iter().find(|p| p.public_port.is_some())?;
    let public = port.public_port?;
    let link = PortLink::of(public, port.private_port);
    let help = door_help(&link, title);
    let arrow = match link {
        PortLink::Open(_) => "↗",
        PortLink::Copy { .. } => "⧉",
    };
    let view = view.clone();
    Some(
        div()
            .id(SharedString::from(format!("card-port-{}", container.id)))
            .px(px(8.))
            .py(px(3.))
            .rounded(px(6.))
            .bg(palette.tint(palette.accent))
            .text_color(palette.link)
            .font_family(palette.mono())
            .text_size(px(11.))
            .cursor_pointer()
            .on_click(move |_, _, cx| {
                cx.stop_propagation();
                follow(&link, &view, cx);
            })
            .child(format!("localhost:{public} {arrow}"))
            .help(help),
    )
}
