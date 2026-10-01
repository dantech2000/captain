//! Starting a session from the source, pumping its output into the emulator, and
//! resizing it.

use std::time::Duration;

use captain_core::EngineError;
use captain_core::model::ExecSession;
use captain_terminal::default_emulator;
use futures::StreamExt;
use gpui_kit::*;

use super::metrics::GridMetrics;
use super::terminal_view::{DEFAULT_SIZE, Live, Phase, TerminalView};

/// How long the grid size must hold still before the session hears about it. Dragging
/// a window edge would otherwise send a resize for every frame.
const RESIZE_DEBOUNCE: Duration = Duration::from_millis(150);
/// Output chunks fed to the emulator before one repaint.
const CHUNKS_PER_FRAME: usize = 64;

impl TerminalView {
    /// Starts a session from the source, if there is one.
    pub(super) fn start(&mut self, cx: &mut Context<Self>) {
        let Some(source) = self.source.clone() else {
            return;
        };
        let (cols, rows) = self
            .metrics
            .map_or(DEFAULT_SIZE, |metrics| (metrics.cols, metrics.rows));
        self.emulator = default_emulator(cols, rows);
        self.phase = Phase::Connecting;
        self.wants_focus = true;
        let open = source.open(cols, rows);
        self.tasks = vec![cx.spawn(async move |this, cx| {
            let result = open.await;
            this.update(cx, |this, cx| this.attach(result, cx)).ok();
        })];
        cx.notify();
    }

    fn attach(&mut self, result: Result<ExecSession, EngineError>, cx: &mut Context<Self>) {
        let session = match result {
            Ok(session) => session,
            Err(error) => {
                self.phase = Phase::Failed(error.to_string());
                cx.notify();
                return;
            }
        };
        self.command = Some(session.command_line());
        let ExecSession {
            input,
            output,
            resizer,
            exit,
            ..
        } = session;
        self.live = Some(Live { input, resizer });
        self.phase = Phase::Running;
        // The grid may have changed size while the exec started.
        let (cols, rows) = self.emulator.size();
        self.schedule_resize(cols, rows, Duration::ZERO, cx);

        let pump = cx.spawn(async move |this, cx| {
            let mut output = output.ready_chunks(CHUNKS_PER_FRAME);
            while let Some(chunks) = output.next().await {
                if this.update(cx, |this, cx| this.feed(chunks, cx)).is_err() {
                    break;
                }
            }
        });
        let wait = cx.spawn(async move |this, cx| {
            let code = exit.await.ok().flatten();
            this.update(cx, |this, cx| this.finish(code, cx)).ok();
        });
        self.tasks.push(pump);
        self.tasks.push(wait);
        cx.notify();
    }

    fn feed(&mut self, chunks: Vec<Result<Vec<u8>, EngineError>>, cx: &mut Context<Self>) {
        for bytes in chunks.into_iter().flatten() {
            self.emulator.feed(&bytes);
        }
        let replies = self.emulator.take_replies();
        if let Some(live) = &self.live {
            live.input.send(replies).ok();
        }
        cx.notify();
    }

    fn finish(&mut self, code: Option<i64>, cx: &mut Context<Self>) {
        let line = match code {
            Some(code) => format!("\r\n[Process exited with code {code}]\r\n"),
            None => "\r\n[Process exited]\r\n".to_string(),
        };
        self.emulator.feed(line.as_bytes());
        self.emulator.scroll_to_bottom();
        self.live = None;
        self.resize_task = None;
        self.phase = Phase::Exited(code);
        cx.notify();
    }

    /// Sends input to the shell and jumps back to the live screen.
    pub(super) fn send(&mut self, bytes: Vec<u8>, cx: &mut Context<Self>) {
        let Some(live) = &self.live else { return };
        live.input.send(bytes).ok();
        self.emulator.select_clear();
        self.emulator.scroll_to_bottom();
        cx.notify();
    }

    /// Called by the grid on each frame. A new size resizes the emulator at once and the
    /// session after a short pause.
    pub(super) fn sync_size(&mut self, metrics: GridMetrics, cx: &mut Context<Self>) {
        self.metrics = Some(metrics);
        let size = (metrics.cols, metrics.rows);
        if size == self.emulator.size() {
            return;
        }
        self.emulator.resize(size.0, size.1);
        self.schedule_resize(size.0, size.1, RESIZE_DEBOUNCE, cx);
        cx.notify();
    }

    fn schedule_resize(&mut self, cols: u16, rows: u16, delay: Duration, cx: &mut Context<Self>) {
        let Some(live) = &self.live else { return };
        let resizer = live.resizer.clone();
        self.resize_task = Some(cx.spawn(async move |_, cx| {
            if !delay.is_zero() {
                cx.background_executor().timer(delay).await;
            }
            if let Err(error) = resizer.resize(cols, rows).await {
                tracing::debug!(%error, "terminal resize failed");
            }
        }));
    }
}
