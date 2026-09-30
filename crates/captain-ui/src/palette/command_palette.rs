use gpui_kit::component::input::{self, InputEvent, InputState};
use gpui_kit::*;

use captain_core::grammar::{Catalog, examples};

use super::command::CommandKind;
use super::keys::{CONTEXT, Complete, Confirm, Dismiss, SelectNext, SelectPrev};
use super::ranking::{self, Ranked};
use super::{commands, footer, grammar, results, search_field, try_row};
use crate::port_forwarding::{ForwardingModel, forwarding_model};
use crate::theme::Palette;
use crate::workspace::Workspace;

/// Search and run commands. The shell shows it over the window; it emits
/// [`DismissEvent`] to close.
pub struct CommandPalette {
    workspace: Entity<Workspace>,
    /// The Kubernetes services for `forward`, read when the palette opens while
    /// k3s runs.
    forwarding: Option<Entity<ForwardingModel>>,
    input: Entity<InputState>,
    /// The highlighted row. Kept in range when the results change.
    selected: usize,
    scroll: ScrollHandle,
    _subscriptions: Vec<Subscription>,
}

impl CommandPalette {
    pub fn new(workspace: Entity<Workspace>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| InputState::new(window, cx).placeholder("Search or run a command"));
        input.update(cx, |input, cx| input.focus(window, cx));
        let edited = cx.subscribe(&input, |this, _, event: &InputEvent, cx| {
            if matches!(event, InputEvent::Change) {
                this.selected = 0;
                this.scroll.scroll_to_item(0);
                cx.notify();
            }
        });
        let mut subscriptions = vec![edited, cx.observe(&workspace, |_, _, cx| cx.notify())];
        let forwarding = grammar::kube_runs(cx).then(|| {
            let model = forwarding_model(cx);
            model.update(cx, |model, cx| model.refresh(cx));
            subscriptions.push(cx.observe(&model, |_, _, cx| cx.notify()));
            model
        });
        Self {
            workspace,
            forwarding,
            input,
            selected: 0,
            scroll: ScrollHandle::new(),
            _subscriptions: subscriptions,
        }
    }

    fn catalog(&self, cx: &App) -> Catalog {
        let forwarding = self.forwarding.as_ref().map(|model| model.read(cx));
        grammar::catalog(self.workspace.read(cx), forwarding, cx)
    }

    /// The rows to show for the current query and workspace: the grammar's
    /// suggestions, then the plain search results unless the line is a command.
    fn results(&self, cx: &App) -> Vec<Ranked> {
        let query = self.input.read(cx).value();
        let workspace = self.workspace.read(cx);
        let palette = Palette::of(cx);
        let mut rows = grammar::rows(&query, &self.catalog(cx), workspace, &palette);
        if !grammar::is_command(&query) {
            let commands = commands::build(workspace, &palette);
            rows.extend(ranking::rank(commands, &query));
        }
        rows
    }

    /// Puts `line` in the search field, with a space for the next word.
    pub(super) fn fill(&mut self, line: &str, window: &mut Window, cx: &mut Context<Self>) {
        let text = format!("{} ", line.trim_end());
        self.input
            .update(cx, |input, cx| input.set_value(text, window, cx));
        self.selected = 0;
        self.scroll.scroll_to_item(0);
        cx.notify();
    }

    fn select_prev(&mut self, _: &SelectPrev, _: &mut Window, cx: &mut Context<Self>) {
        self.step(-1, cx);
    }

    fn select_next(&mut self, _: &SelectNext, _: &mut Window, cx: &mut Context<Self>) {
        self.step(1, cx);
    }

    /// Moves the highlight by `delta` rows, wrapping at both ends.
    fn step(&mut self, delta: isize, cx: &mut Context<Self>) {
        let results = self.results(cx);
        let len = results.len() as isize;
        if len == 0 {
            return;
        }
        let current = (self.selected as isize).min(len - 1);
        self.selected = (current + delta).rem_euclid(len) as usize;
        self.scroll
            .scroll_to_item(ranking::child_index(&results, self.selected));
        cx.notify();
    }

    fn confirm(&mut self, _: &Confirm, window: &mut Window, cx: &mut Context<Self>) {
        self.run_at(self.selected, window, cx);
    }

    /// Tab: puts the highlighted grammar row in the search field.
    fn complete(&mut self, _: &Complete, window: &mut Window, cx: &mut Context<Self>) {
        let completion = self
            .results(cx)
            .into_iter()
            .nth(self.selected)
            .and_then(|ranked| ranked.command.completion);
        if let Some(line) = completion {
            self.fill(&line, window, cx);
        }
    }

    fn dismiss(&mut self, _: &Dismiss, _: &mut Window, cx: &mut Context<Self>) {
        cx.emit(DismissEvent);
    }

    /// Runs the command in row `ix` and closes the palette. A command that needs
    /// more words goes into the search field instead.
    pub(super) fn run_at(&mut self, ix: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(ranked) = self.results(cx).into_iter().nth(ix) else {
            return;
        };
        if let CommandKind::Complete(line) = &ranked.command.kind {
            self.fill(line, window, cx);
            return;
        }
        ranked.command.kind.run(&self.workspace, cx);
        cx.emit(DismissEvent);
    }

    /// Highlights row `ix` when the mouse moves over it.
    pub(super) fn hover(&mut self, ix: usize, cx: &mut Context<Self>) {
        if self.selected != ix {
            self.selected = ix;
            cx.notify();
        }
    }
}

impl EventEmitter<DismissEvent> for CommandPalette {}

impl Focusable for CommandPalette {
    /// The search field, so typing goes straight into the query.
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.input.read(cx).focus_handle(cx)
    }
}

impl Render for CommandPalette {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = Palette::of(cx);
        let query = self.input.read(cx).value();
        let results = self.results(cx);
        self.selected = self.selected.min(results.len().saturating_sub(1));
        let catalog = self.catalog(cx);
        let error = grammar::error(&query, &catalog);
        let tries = query.trim().is_empty().then(|| examples(&catalog));

        div()
            .id("command-palette")
            .key_context(CONTEXT)
            .on_action(cx.listener(Self::select_prev))
            .on_action(cx.listener(Self::select_next))
            .on_action(cx.listener(Self::confirm))
            .on_action(cx.listener(Self::complete))
            .on_action(cx.listener(Self::dismiss))
            // The search field handles escape itself and then passes it up.
            .on_action(cx.listener(|_, _: &input::Escape, _, cx| cx.emit(DismissEvent)))
            .on_mouse_down_out(cx.listener(|_, _, _, cx| cx.emit(DismissEvent)))
            .w(px(640.))
            .flex()
            .flex_col()
            .overflow_hidden()
            .rounded(px(16.))
            .bg(palette.panel)
            .border_1()
            .border_color(palette.sep)
            .shadow(vec![BoxShadow {
                color: hsla(0., 0., 0., if palette.dark { 0.55 } else { 0.25 }),
                offset: point(px(0.), px(30.)),
                blur_radius: px(80.),
                spread_radius: px(0.),
                inset: false,
            }])
            .text_size(px(13.))
            .text_color(palette.text)
            .child(search_field::render(&self.input, &palette))
            .children(tries.map(|tries| try_row::render(tries, &palette, cx)))
            .child(results::render(
                &results,
                self.selected,
                &query,
                error,
                &self.scroll,
                &palette,
                cx,
            ))
            .child(footer::render(&palette))
    }
}
