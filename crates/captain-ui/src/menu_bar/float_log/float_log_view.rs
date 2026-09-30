//! The floating log: the container's name and project, a health strip, and the
//! newest lines of its output as they arrive.

use std::collections::VecDeque;
use std::time::Duration;

use captain_core::model::{ContainerState, Health, LogLevel, LogLine};
use futures::StreamExt;
use gpui_kit::*;

use crate::help::HelpExt;
use crate::theme::Palette;
use crate::widgets::drag_region;
use crate::workspace::Workspace;

/// Past lines to load when the window opens.
const TAIL: usize = 40;
/// The window keeps this many lines; it shows as many as fit.
const KEEP: usize = 60;
/// Bars in the health strip, one per sample.
const BEATS: usize = 20;
const BEAT_EVERY: Duration = Duration::from_secs(3);

/// One health sample.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Beat {
    Good,
    Warn,
    Bad,
}

pub struct FloatLogView {
    workspace: Entity<Workspace>,
    id: String,
    name: String,
    project: Option<String>,
    lines: VecDeque<LogLine>,
    beats: VecDeque<Beat>,
    stream_error: Option<String>,
    _tasks: Vec<Task<()>>,
}

impl FloatLogView {
    pub fn new(
        workspace: Entity<Workspace>,
        id: String,
        name: String,
        project: Option<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        window.set_window_title(&match &project {
            Some(project) => format!("{name} · {project}"),
            None => name.clone(),
        });
        let mut tasks = Vec::new();
        if let Some(engine) = workspace.read(cx).engine() {
            let mut stream = engine.logs(&id, TAIL);
            tasks.push(cx.spawn(async move |this, cx| {
                while let Some(line) = stream.next().await {
                    let more = this.update(cx, |this, cx| {
                        match line {
                            Ok(line) => this.push(line),
                            Err(error) => {
                                this.stream_error =
                                    Some(format!("The log stream stopped: {error}"));
                            }
                        }
                        cx.notify();
                        this.stream_error.is_none()
                    });
                    if !more.unwrap_or(false) {
                        break;
                    }
                }
            }));
        }
        tasks.push(cx.spawn(async move |this, cx| {
            loop {
                if this.update(cx, |this, cx| this.sample(cx)).is_err() {
                    return;
                }
                cx.background_executor().timer(BEAT_EVERY).await;
            }
        }));
        Self {
            workspace,
            id,
            name,
            project,
            lines: VecDeque::new(),
            beats: VecDeque::new(),
            stream_error: None,
            _tasks: tasks,
        }
    }

    fn push(&mut self, line: LogLine) {
        if self.lines.len() == KEEP {
            self.lines.pop_front();
        }
        self.lines.push_back(line);
    }

    /// Records the container's state as one bar of the health strip.
    fn sample(&mut self, cx: &mut Context<Self>) {
        let beat = self.current_beat(cx);
        if self.beats.len() == BEATS {
            self.beats.pop_front();
        }
        self.beats.push_back(beat);
        cx.notify();
    }

    fn current_beat(&self, cx: &App) -> Beat {
        let workspace = self.workspace.read(cx);
        match workspace.store().find(&self.id) {
            Some(c) if c.state == ContainerState::Running => match c.health {
                Some(Health::Unhealthy) => Beat::Bad,
                Some(Health::Starting) => Beat::Warn,
                _ => Beat::Good,
            },
            Some(c) if c.state == ContainerState::Restarting => Beat::Warn,
            _ => Beat::Bad,
        }
    }
}

impl Render for FloatLogView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = Palette::of(cx);
        let color = |beat: Beat| match beat {
            Beat::Good => palette.green,
            Beat::Warn => palette.orange,
            Beat::Bad => palette.red,
        };
        let now = color(self.current_beat(cx));
        let subtitle = match &self.project {
            Some(project) => format!("{project} · stays on top"),
            None => "stays on top".to_string(),
        };
        let strip = div()
            .id("float-log-health")
            .flex()
            .gap(px(2.))
            .children(
                self.beats
                    .iter()
                    .map(|beat| div().w(px(4.)).h(px(12.)).rounded(px(1.)).bg(color(*beat))),
            )
            .help(format!(
                "The state of {} every few seconds: green runs, amber restarts, red stopped or unhealthy.",
                self.name
            ));
        let header = drag_region("float-log-title")
            .h(px(34.))
            .flex_shrink_0()
            .flex()
            .items_center()
            .gap(px(8.))
            .pl(px(72.))
            .pr(px(12.))
            .border_b_1()
            .border_color(palette.sep)
            .child(div().size(px(7.)).rounded_full().bg(now))
            .child(
                div()
                    .text_size(px(12.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(self.name.clone()),
            )
            .child(
                div()
                    .text_size(px(11.))
                    .text_color(palette.text3)
                    .child(subtitle),
            )
            .child(div().flex_1())
            .child(strip);
        let line_color = |line: &LogLine| match line.level {
            LogLevel::Info => palette.text2,
            LogLevel::Warn => palette.warn_text,
            LogLevel::Error => palette.red,
        };
        let body = div()
            .flex_1()
            .min_h_0()
            .overflow_hidden()
            .flex()
            .flex_col()
            .justify_end()
            .px(px(12.))
            .py(px(8.))
            .font_family(palette.mono())
            .text_size(px(11.))
            .line_height(px(19.))
            .children(self.lines.iter().map(|line| {
                div()
                    .flex_shrink_0()
                    .truncate()
                    .text_color(line_color(line))
                    .child(line.text.clone())
            }))
            .children(
                self.stream_error
                    .clone()
                    .map(|error| div().flex_shrink_0().text_color(palette.red).child(error)),
            );
        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(palette.terminal)
            .text_color(palette.text)
            .child(header)
            .child(body)
    }
}
