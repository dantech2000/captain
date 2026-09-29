use std::rc::Rc;
use std::time::Duration;

use captain_core::EngineStream;
use captain_core::model::LogLine;
use captain_core::store::{LevelFilter, LogBuffer, find_matches};
use futures::StreamExt;
use gpui_kit::component::input::{InputEvent, InputState};
use gpui_kit::*;

use super::jump_pill::jump_pill;
use super::local_offset::local_offset;
use super::log_list::log_list;
use super::toolbar;
use crate::theme::Palette;

/// How long the Copy button shows a check after a copy.
const COPIED_FOR: Duration = Duration::from_millis(1500);

/// The Logs tab of one container: its recent lines and how the tab shows them.
pub struct LogsPane {
    buffer: LogBuffer,
    level: LevelFilter,
    query: SharedString,
    /// The search field. It needs a window, so the first render creates it.
    search: Option<Entity<InputState>>,
    show_time: bool,
    /// Whether the list sticks to the newest line.
    following: bool,
    /// Lines that passed the filters since following paused.
    unseen: usize,
    scroll: UniformListScrollHandle,
    stream_task: Option<Task<()>>,
    /// Set for a moment after a copy.
    copied: Option<Task<()>>,
    _subscriptions: Vec<Subscription>,
}

impl Default for LogsPane {
    fn default() -> Self {
        Self {
            buffer: LogBuffer::default(),
            level: LevelFilter::default(),
            query: SharedString::default(),
            search: None,
            show_time: true,
            following: true,
            unseen: 0,
            scroll: UniformListScrollHandle::new(),
            stream_task: None,
            copied: None,
            _subscriptions: Vec::new(),
        }
    }
}

impl LogsPane {
    /// Empties the view and shows the lines of `lines` as they arrive. The filters
    /// and search text stay.
    pub fn load(&mut self, mut lines: EngineStream<LogLine>, cx: &mut Context<Self>) {
        self.buffer.clear();
        self.following = true;
        self.unseen = 0;
        self.stream_task = Some(cx.spawn(async move |this, cx| {
            while let Some(Ok(line)) = lines.next().await {
                if this.update(cx, |this, cx| this.push(line, cx)).is_err() {
                    break;
                }
            }
        }));
        cx.notify();
    }

    fn push(&mut self, line: LogLine, cx: &mut Context<Self>) {
        if !self.following && self.shows(&line) {
            self.unseen += 1;
        }
        self.buffer.push(line);
        cx.notify();
    }

    /// Whether `line` passes the level filter and the search text.
    fn shows(&self, line: &LogLine) -> bool {
        self.level.matches(line) && find_matches(&line.text, &self.query).is_some()
    }

    pub(super) fn level(&self) -> LevelFilter {
        self.level
    }

    pub(super) fn show_time(&self) -> bool {
        self.show_time
    }

    pub(super) fn following(&self) -> bool {
        self.following
    }

    pub(super) fn copied(&self) -> bool {
        self.copied.is_some()
    }

    pub(super) fn total(&self) -> usize {
        self.buffer.len()
    }

    pub(super) fn set_level(&mut self, level: LevelFilter, cx: &mut Context<Self>) {
        self.level = level;
        cx.notify();
    }

    pub(super) fn toggle_time(&mut self, cx: &mut Context<Self>) {
        self.show_time = !self.show_time;
        cx.notify();
    }

    /// Pauses or resumes following. Resuming jumps to the newest line.
    pub(super) fn set_following(&mut self, following: bool, cx: &mut Context<Self>) {
        self.following = following;
        self.unseen = 0;
        cx.notify();
    }

    /// Empties the view. The container keeps its logs.
    pub(super) fn clear(&mut self, cx: &mut Context<Self>) {
        self.buffer.clear();
        self.unseen = 0;
        cx.notify();
    }

    /// Copies the lines that pass the filters, with their times when the time column shows.
    pub(super) fn copy_visible(&mut self, cx: &mut Context<Self>) {
        let offset = self.clock_offset();
        let text = self
            .buffer
            .search(self.level, &self.query)
            .iter()
            .map(|found| found.line.copy_text(offset))
            .collect::<Vec<_>>()
            .join("\n");
        self.copy_to_clipboard(text, cx);
    }

    pub(super) fn copy_to_clipboard(&mut self, text: String, cx: &mut Context<Self>) {
        cx.write_to_clipboard(ClipboardItem::new_string(text));
        self.copied = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(COPIED_FOR).await;
            this.update(cx, |this, cx| {
                this.copied = None;
                cx.notify();
            })
            .ok();
        }));
        cx.notify();
    }

    /// Scrolling up pauses following. Scrolling back to the end resumes it.
    fn on_scroll(&mut self, delta: Pixels, cx: &mut Context<Self>) {
        if delta > px(0.) && self.following && self.scroll.is_scrollable() {
            self.set_following(false, cx);
        } else if delta < px(0.)
            && !self.following
            && self.scroll.is_scrolled_to_end() == Some(true)
        {
            self.set_following(true, cx);
        }
    }

    fn clock_offset(&self) -> Option<i32> {
        self.show_time.then(local_offset)
    }

    fn search_input(&mut self, window: &mut Window, cx: &mut Context<Self>) -> Entity<InputState> {
        if let Some(input) = &self.search {
            return input.clone();
        }
        let input = cx.new(|cx| InputState::new(window, cx).placeholder("Search logs"));
        let changes = cx.subscribe(&input, |this, input, event: &InputEvent, cx| {
            if let InputEvent::Change = event {
                this.query = input.read(cx).value();
                cx.notify();
            }
        });
        self._subscriptions.push(changes);
        self.search = Some(input.clone());
        input
    }
}

impl Render for LogsPane {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = Palette::of(cx);
        let search = self.search_input(window, cx);
        let matches = Rc::new(self.buffer.search(self.level, &self.query));
        let count = matches.len();
        if self.following && count > 0 {
            self.scroll
                .scroll_to_item(count - 1, ScrollStrategy::Bottom);
        }
        let list = log_list(
            matches,
            self.clock_offset(),
            &self.scroll,
            &palette,
            cx.entity().downgrade(),
        );
        let jump = (!self.following).then(|| {
            let resume = cx.listener(|this, _, _, cx| this.set_following(true, cx));
            div()
                .absolute()
                .bottom(px(12.))
                .left_0()
                .right_0()
                .flex()
                .justify_center()
                .child(jump_pill(self.unseen, &palette, resume))
        });

        div()
            .flex_1()
            .min_h_0()
            .flex()
            .flex_col()
            .gap(px(10.))
            .px(px(20.))
            .pt(px(14.))
            .pb(px(20.))
            .child(toolbar::search_row(self, &search, &palette, cx))
            .child(toolbar::filter_row(self, count, &palette, cx))
            .child(
                div()
                    .relative()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .flex_col()
                    .py(px(8.))
                    .rounded(px(10.))
                    .bg(palette.terminal)
                    .border_1()
                    .border_color(palette.sep)
                    .font_family(palette.mono())
                    .text_size(px(11.))
                    .on_scroll_wheel(cx.listener(|this, event: &ScrollWheelEvent, window, cx| {
                        let delta = event.delta.pixel_delta(window.line_height());
                        this.on_scroll(delta.y, cx);
                    }))
                    .child(list)
                    .children(jump),
            )
    }
}
