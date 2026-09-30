use std::collections::HashMap;
use std::rc::Rc;

use captain_core::model::{ContainerState, EngineEvent, EventKind, LogLine, LogOptions, LogStream};
use captain_core::store::{GroupKey, LogCursor, ProjectLog, ProjectLogEntry};
use futures::StreamExt;
use gpui_kit::*;

use super::group_info::{is_sandbox, service_name};
use super::log_rows;
use crate::theme::Palette;
use crate::workspace::Workspace;

/// How many past lines each container sends when the page opens.
const FIRST_TAIL: usize = 100;

/// One log for all services of the shown entry. It follows each running container
/// with its own stream, and opens a new stream when a container starts again, so
/// the lines outlive restarts.
pub struct ProjectLogView {
    workspace: Entity<Workspace>,
    focus: Option<GroupKey>,
    log: ProjectLog,
    /// The stream task of each followed container, with its number.
    streams: HashMap<String, (u64, Task<()>)>,
    next_stream: u64,
    /// The time of each container's last `start` event.
    started: HashMap<String, i64>,
    /// Where each container's last stream stopped.
    cursors: HashMap<String, LogCursor>,
    /// True once the first streams of this entry opened. Later containers are new,
    /// so their whole output belongs in the log.
    opened: bool,
    /// The entries as the list shows them, rebuilt on the next render after a change.
    rows: Rc<Vec<ProjectLogEntry>>,
    dirty: bool,
    /// Whether the list sticks to the newest entry. Scrolling up pauses it.
    following: bool,
    scroll: UniformListScrollHandle,
    _subscriptions: Vec<Subscription>,
}

impl ProjectLogView {
    pub fn new(workspace: Entity<Workspace>, cx: &mut Context<Self>) -> Self {
        let subscriptions = vec![
            cx.observe(&workspace, |this, _, cx| this.follow(cx)),
            cx.subscribe(&workspace, |this, _, event: &EngineEvent, cx| {
                this.record(event, cx)
            }),
        ];
        let mut view = Self {
            workspace,
            focus: None,
            log: ProjectLog::default(),
            streams: HashMap::new(),
            next_stream: 0,
            started: HashMap::new(),
            cursors: HashMap::new(),
            opened: false,
            rows: Rc::default(),
            dirty: false,
            following: true,
            scroll: UniformListScrollHandle::new(),
            _subscriptions: subscriptions,
        };
        view.follow(cx);
        view
    }

    /// Starts over for a new entry, and opens a stream for each running container
    /// that has none.
    fn follow(&mut self, cx: &mut Context<Self>) {
        let focus = self.workspace.read(cx).focus().cloned();
        if focus != self.focus {
            self.focus = focus;
            self.log.clear();
            self.streams.clear();
            self.started.clear();
            self.cursors.clear();
            self.opened = false;
            self.changed(cx);
        }
        let workspace = self.workspace.read(cx);
        let (Some(group), Some(engine)) = (workspace.focused_group(), workspace.engine()) else {
            return;
        };
        let opened = self.opened;
        for container in group.containers.iter().filter(|c| !is_sandbox(c)) {
            if container.state != ContainerState::Running
                || self.streams.contains_key(&container.id)
            {
                continue;
            }
            // A restart continues from the newest line seen, or else its start; a
            // new container sends everything. The cursor drops the replayed lines.
            let since = self
                .cursors
                .get(&container.id)
                .and_then(LogCursor::since)
                .or_else(|| self.started.get(&container.id).copied())
                .or(opened.then_some(container.created));
            let options = LogOptions {
                tail: since.is_none().then_some(FIRST_TAIL),
                since,
                follow: true,
            };
            let mut lines = engine.logs_with(&container.id, options);
            let (id, service) = (container.id.clone(), service_name(container));
            self.next_stream += 1;
            let number = self.next_stream;
            let task = cx.spawn(async move |this, cx| {
                while let Some(Ok(line)) = lines.next().await {
                    let pushed = this.update(cx, |this, cx| {
                        if this.cursors.entry(id.clone()).or_default().advance(&line) {
                            this.log.push_line(&service, line);
                            this.changed(cx);
                        }
                    });
                    if pushed.is_err() {
                        return;
                    }
                }
                // The stream ends when the container stops. Forget it, so a start
                // opens a new one.
                this.update(cx, |this, _| {
                    if this.streams.get(&id).is_some_and(|(n, _)| *n == number)
                        && let Some((_, task)) = this.streams.remove(&id)
                    {
                        task.detach();
                    }
                })
                .ok();
            });
            self.streams.insert(container.id.clone(), (number, task));
        }
        self.opened = true;
    }

    /// Adds exit dividers and remembers start times for the shown containers.
    fn record(&mut self, event: &EngineEvent, cx: &mut Context<Self>) {
        if event.kind != EventKind::Container {
            return;
        }
        let workspace = self.workspace.read(cx);
        let Some(group) = workspace.focused_group() else {
            return;
        };
        let Some(container) = group
            .containers
            .iter()
            .find(|c| c.id == event.id && !is_sandbox(c))
        else {
            return;
        };
        if event.action == "start"
            && let Some(time) = event.time
        {
            self.started.insert(event.id.clone(), time);
        }
        let before = self.log.len();
        self.log.record(&service_name(container), event);
        if self.log.len() != before {
            self.changed(cx);
        }
    }

    /// Adds the output of a Compose command, one row per line, under `compose`.
    pub fn push_output(&mut self, text: &str, cx: &mut Context<Self>) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |since| since.as_secs() as i64);
        for line in text.lines().filter(|line| !line.trim().is_empty()) {
            let mut line = LogLine::new(LogStream::Stderr, line);
            line.timestamp = Some(now);
            self.log.push_line("compose", line);
        }
        self.changed(cx);
    }

    fn changed(&mut self, cx: &mut Context<Self>) {
        self.dirty = true;
        cx.notify();
    }

    /// Scrolling up pauses following. Scrolling back to the end resumes it.
    fn on_scroll(&mut self, delta: Pixels, cx: &mut Context<Self>) {
        let following = if delta > px(0.) {
            !self.scroll.is_scrollable()
        } else {
            self.following || self.scroll.is_scrolled_to_end() == Some(true)
        };
        if following != self.following {
            self.following = following;
            cx.notify();
        }
    }
}

impl Render for ProjectLogView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = Palette::of(cx);
        let workspace = self.workspace.read(cx);
        let services: Vec<String> = workspace
            .focused_group()
            .map(|group| {
                let mut names: Vec<String> = group
                    .containers
                    .iter()
                    .filter(|c| !is_sandbox(c))
                    .map(service_name)
                    .collect();
                names.sort();
                names.dedup();
                names
            })
            .unwrap_or_default();
        if self.dirty {
            self.rows = Rc::new(self.log.entries().cloned().collect());
            self.dirty = false;
        }
        if self.following && !self.rows.is_empty() {
            self.scroll
                .scroll_to_item(self.rows.len() - 1, ScrollStrategy::Bottom);
        }
        log_rows::render(
            self.rows.clone(),
            services,
            self.following,
            &self.scroll,
            &palette,
        )
        .on_scroll_wheel(cx.listener(|this, event: &ScrollWheelEvent, window, cx| {
            let delta = event.delta.pixel_delta(window.line_height());
            this.on_scroll(delta.y, cx);
        }))
    }
}
