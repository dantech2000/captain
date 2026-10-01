use captain_core::model::{EngineEvent, EventKind};
use gpui_kit::assets::IconName;
use gpui_kit::component::Icon;
use gpui_kit::*;

use super::latest_event::LatestEvent;
use super::segments::{Segment, agent, disk, segments};
use crate::agents::{activity_time, agent_activity, latest_activity};
use crate::engine_host::{host_model, summary as host_summary};
use crate::help::{self, HelpExt, Hint};
use crate::kubernetes::kubernetes_model;
use crate::palette::key_hint;
use crate::settings::engine_source;
use crate::storage::storage_model;
use crate::theme::Palette;
use crate::workspace::{Connection, Workspace};

/// The bar at the bottom of the main window. On the left, the help sentence of the
/// control under the mouse, else the latest notable event. On the right, the engine,
/// its use, Kubernetes, and the docker context. See feature 0029.
pub struct StatusBar {
    workspace: Entity<Workspace>,
    latest: LatestEvent,
    /// The docker CLI's current context, read again when the engine endpoint changes.
    context: Option<String>,
    endpoint: Option<String>,
    _subscriptions: Vec<Subscription>,
}

impl StatusBar {
    pub fn new(workspace: Entity<Workspace>, cx: &mut Context<Self>) -> Self {
        let help = help::ensure(cx);
        let mut subscriptions = vec![
            cx.observe(&help, |_, _, cx| cx.notify()),
            cx.observe(&workspace, |this, workspace, cx| {
                this.follow_endpoint(&workspace, cx);
                cx.notify();
            }),
            cx.subscribe(&workspace, Self::on_engine_event),
        ];
        subscriptions.extend(host_model(cx).map(|host| cx.observe(&host, |_, _, cx| cx.notify())));
        subscriptions
            .extend(kubernetes_model(cx).map(|model| cx.observe(&model, |_, _, cx| cx.notify())));
        subscriptions
            .extend(storage_model(cx).map(|model| cx.observe(&model, |_, _, cx| cx.notify())));
        subscriptions
            .extend(agent_activity(cx).map(|watch| cx.observe(&watch, |_, _, cx| cx.notify())));
        let mut bar = Self {
            workspace: workspace.clone(),
            latest: LatestEvent::default(),
            context: None,
            endpoint: None,
            _subscriptions: subscriptions,
        };
        bar.follow_endpoint(&workspace, cx);
        bar
    }

    fn follow_endpoint(&mut self, workspace: &Entity<Workspace>, cx: &App) {
        let endpoint = match workspace.read(cx).connection() {
            Connection::Connected(info) => Some(info.endpoint.clone()),
            _ => None,
        };
        if endpoint == self.endpoint {
            return;
        }
        self.endpoint = endpoint;
        self.context = engine_source(cx).map(|source| {
            source
                .contexts()
                .current
                .unwrap_or_else(|| "default".to_string())
        });
    }

    fn on_engine_event(
        &mut self,
        workspace: Entity<Workspace>,
        event: &EngineEvent,
        cx: &mut Context<Self>,
    ) {
        if event.kind != EventKind::Container {
            return;
        }
        let store = workspace.read(cx).store();
        if store.is_hidden(&event.id, event.name.as_deref()) {
            return;
        }
        let name = store
            .find(&event.id)
            .map(|container| container.name.clone())
            .unwrap_or_else(|| event.id.chars().take(12).collect());
        if self
            .latest
            .record(&event.action, &event.id, &name, chrono::Local::now())
        {
            cx.notify();
        }
    }

    fn left(&self, hint: Option<Hint>, palette: &Palette) -> Div {
        let row = div().flex_1().min_w_0().flex().items_center().gap(px(8.));
        if let Some(hint) = hint {
            return row
                .child(
                    Icon::new(IconName::Info)
                        .size(px(13.))
                        .text_color(palette.accent_fg),
                )
                .child(
                    div()
                        .min_w_0()
                        .truncate()
                        .text_color(palette.text)
                        .child(hint.text),
                )
                .children(hint.keys.iter().map(|key| key_hint(*key, palette)));
        }
        let (dot, line) = match self.latest.line() {
            Some(line) => (palette.red, format!("{line} · hover anything for help")),
            None => (palette.gray, "Ready · hover anything for help".to_string()),
        };
        row.child(div().size(px(7.)).flex_shrink_0().rounded_full().bg(dot))
            .child(div().min_w_0().truncate().child(line))
    }
}

/// The Disk segment, once the storage model has read the disk use.
fn disk_segment(cx: &App) -> Option<Segment> {
    let model = storage_model(cx)?;
    let model = model.read(cx);
    let breakdown = model.breakdown(cx)?;
    Some(disk(
        breakdown.used,
        breakdown.capacity,
        model.default_bytes(),
    ))
}

fn segment(segment: Segment, palette: &Palette) -> Stateful<Div> {
    let hover = palette.nav_selected;
    div()
        .id(segment.id)
        .h(px(22.))
        .px(px(8.))
        .flex_shrink_0()
        .flex()
        .items_center()
        .gap(px(6.))
        .rounded(px(6.))
        .whitespace_nowrap()
        .hover(move |style| style.bg(hover))
        .children(
            segment
                .dot
                .map(|dot| div().size(px(6.)).rounded_full().bg(dot)),
        )
        .child(segment.label)
        .help(segment.help)
}

impl Render for StatusBar {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = Palette::of(cx);
        let hint = help::hover_help(cx).and_then(|help| help.read(cx).hint().cloned());
        let host = host_summary(cx);
        let kubernetes = host
            .as_ref()
            .and(kubernetes_model(cx))
            .map(|model| model.read(cx).status().clone());
        let mut right = segments(
            self.workspace.read(cx),
            host.as_ref(),
            kubernetes.as_ref(),
            self.context.as_deref(),
            &palette,
        );
        if let Some(segment) = disk_segment(cx) {
            let at = right
                .iter()
                .position(|s| s.id == "status-memory")
                .map_or(right.len().min(1), |ix| ix + 1);
            right.insert(at, segment);
        }
        if let Some(entry) = latest_activity(cx) {
            right.insert(0, agent(&entry, &activity_time(entry.at), &palette));
        }
        div()
            .h(px(30.))
            .flex_shrink_0()
            .flex()
            .items_center()
            .gap(px(4.))
            .pl(px(14.))
            .pr(px(10.))
            .bg(palette.side)
            .border_t_1()
            .border_color(palette.sep)
            .text_size(px(11.5))
            .text_color(palette.text2)
            .child(self.left(hint, &palette))
            .children(right.into_iter().map(|item| segment(item, &palette)))
    }
}
