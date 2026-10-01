use std::rc::Rc;
use std::sync::Arc;

use captain_core::Engine;
use gpui_kit::*;

use super::{header, status};
use crate::terminal::{EndedBar, ExecSource, TerminalView};
use crate::theme::Palette;

/// The container a terminal belongs to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalTarget {
    pub id: String,
    pub name: String,
    pub running: bool,
}

/// The Terminal tab: one shell in the selected container, through the engine's
/// exec. It starts the session only when the tab is shown.
pub struct TerminalPane {
    pub(super) target: Option<TerminalTarget>,
    pub(super) view: Entity<TerminalView>,
}

impl TerminalPane {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let view = cx.new(|cx| TerminalView::new(EndedBar::Note, cx));
        cx.observe(&view, |_, _, cx| cx.notify()).detach();
        Self { target: None, view }
    }

    /// Follows the inspector's container. A different container, or a stop, ends the
    /// session; the tab starts a new one when it is shown.
    pub fn set_target(
        &mut self,
        target: Option<TerminalTarget>,
        engine: Option<Arc<dyn Engine>>,
        cx: &mut Context<Self>,
    ) {
        let same = target.as_ref().map(|t| &t.id) == self.target.as_ref().map(|t| &t.id);
        let running = target.as_ref().is_some_and(|t| t.running);
        if !same || !running || !self.running() {
            let source = match (&target, engine) {
                (Some(target), Some(engine)) if target.running => Some(Rc::new(ExecSource {
                    engine,
                    container: target.id.clone(),
                }) as Rc<_>),
                _ => None,
            };
            self.view.update(cx, |view, cx| view.set_source(source, cx));
        }
        self.target = target;
        cx.notify();
    }

    /// The tab is showing: start a session if there is none.
    pub fn show(&mut self, cx: &mut Context<Self>) {
        self.view.update(cx, |view, cx| view.show(cx));
    }

    pub(super) fn running(&self) -> bool {
        self.target.as_ref().is_some_and(|t| t.running)
    }
}

impl Render for TerminalPane {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = Palette::of(cx);
        let body = if self.running() {
            self.view.clone().into_any_element()
        } else {
            status::not_running(&palette).into_any_element()
        };

        div()
            .flex_1()
            .min_h_0()
            .flex()
            .flex_col()
            .gap(px(10.))
            .px(px(20.))
            .pt(px(14.))
            .pb(px(20.))
            .child(header::render(self, &palette, cx))
            .child(body)
    }
}
