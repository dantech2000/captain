//! Keyboard, clipboard, mouse, and scroll wheel handling for the grid.

use captain_terminal::{Key, KeyInput, SelectionKind};
use gpui_kit::*;

use super::grid::{LINE_HEIGHT, TerminalGrid};
use super::input::{Shortcut, key_input, shortcut};
use super::keys::KEY_CONTEXT;
use super::terminal_view::{Phase, TerminalView};
use crate::theme::Palette;

const MACOS: bool = cfg!(target_os = "macos");

impl TerminalView {
    /// The rounded box around the grid, with every input handler, and an overlay while
    /// the session is not running.
    pub(super) fn grid_box(
        &self,
        grid: TerminalGrid,
        palette: &Palette,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        div()
            .id("terminal-grid")
            .relative()
            .flex_1()
            .min_h_0()
            .p(px(8.))
            .rounded(px(10.))
            .bg(palette.terminal)
            .border_1()
            .border_color(palette.sep)
            .overflow_hidden()
            .cursor(CursorStyle::IBeam)
            .track_focus(&self.focus)
            .key_context(KEY_CONTEXT)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _, cx| {
                if this.on_key(&event.keystroke, cx) {
                    cx.stop_propagation();
                }
            }))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, event: &MouseDownEvent, window, cx| {
                    window.focus(&this.focus, cx);
                    this.select_start(event, cx);
                }),
            )
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _, cx| {
                if event.pressed_button == Some(MouseButton::Left) {
                    this.select_update(event.position, cx);
                }
            }))
            .on_scroll_wheel(cx.listener(|this, event: &ScrollWheelEvent, _, cx| {
                this.on_scroll(event.delta.pixel_delta(LINE_HEIGHT).y, cx);
            }))
            .child(grid)
            .children(super::overlay::render(self, palette, cx))
    }

    /// Handles a keystroke. Returns false for keystrokes the app should see.
    fn on_key(&mut self, keystroke: &Keystroke, cx: &mut Context<Self>) -> bool {
        match shortcut(keystroke, MACOS) {
            Some(Shortcut::Copy) => {
                if let Some(text) = self.emulator.selection_text() {
                    cx.write_to_clipboard(ClipboardItem::new_string(text));
                }
                return true;
            }
            Some(Shortcut::Paste) => {
                let text = cx.read_from_clipboard().and_then(|item| item.text());
                if let Some(text) = text {
                    let bytes = self.emulator.encode_paste(&text);
                    self.send(bytes, cx);
                }
                return true;
            }
            None => {}
        }
        let Some(input) = key_input(keystroke, MACOS) else {
            return false;
        };
        if self.phase != Phase::Running {
            return false;
        }
        if let Some(bytes) = self.emulator.encode_key(input) {
            self.send(bytes, cx);
        }
        true
    }

    fn select_start(&mut self, event: &MouseDownEvent, cx: &mut Context<Self>) {
        let Some(metrics) = self.metrics else { return };
        let kind = match event.click_count {
            2 => SelectionKind::Word,
            n if n >= 3 => SelectionKind::Line,
            _ => SelectionKind::Simple,
        };
        self.emulator
            .select_start(metrics.point_at(event.position), kind);
        cx.notify();
    }

    fn select_update(&mut self, position: Point<Pixels>, cx: &mut Context<Self>) {
        let Some(metrics) = self.metrics else { return };
        self.emulator.select_update(metrics.point_at(position));
        cx.notify();
    }

    /// Scrolls the history. On the alternate screen, where `less` or `vim` runs, the
    /// wheel sends arrow keys instead, as other terminals do.
    fn on_scroll(&mut self, delta: Pixels, cx: &mut Context<Self>) {
        self.scroll_rest += delta;
        let lines = (self.scroll_rest / LINE_HEIGHT).trunc() as i32;
        if lines == 0 {
            return;
        }
        self.scroll_rest -= LINE_HEIGHT * lines as f32;
        let modes = self.emulator.modes();
        if modes.alt_screen && modes.alternate_scroll {
            let key = if lines > 0 { Key::Up } else { Key::Down };
            let bytes: Vec<u8> = (0..lines.unsigned_abs())
                .filter_map(|_| self.emulator.encode_key(KeyInput::new(key)))
                .flatten()
                .collect();
            if let Some(live) = &self.live {
                live.input.send(bytes).ok();
            }
            return;
        }
        self.emulator.scroll(lines);
        cx.notify();
    }
}
