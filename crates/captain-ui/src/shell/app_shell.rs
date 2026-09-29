use std::sync::Arc;

use captain_core::{Engine, EngineError};
use gpui_kit::component::{Theme, h_flex};
use gpui_kit::*;

use super::engine_status::EngineStatus;
use super::page::Page;
use super::sidebar;
use crate::containers::ContainersView;

/// Connects to an engine. It may block, so the shell runs it on a background thread.
pub type Connector = Box<dyn FnOnce() -> Result<Arc<dyn Engine>, EngineError> + Send>;

/// The root view: the sidebar and the active page.
pub struct AppShell {
    page: Page,
    status: EngineStatus,
    containers: Entity<ContainersView>,
    _appearance: Subscription,
}

impl AppShell {
    pub fn new(connect: Connector, window: &mut Window, cx: &mut Context<Self>) -> Self {
        Theme::sync_system_appearance(Some(window), cx);
        let appearance = window.observe_window_appearance(|window, cx| {
            Theme::sync_system_appearance(Some(window), cx)
        });

        let containers = cx.new(|cx| ContainersView::new(window, cx));
        let shell = Self {
            page: Page::Containers,
            status: EngineStatus::Connecting,
            containers,
            _appearance: appearance,
        };
        shell.connect(connect, cx);
        shell
    }

    fn connect(&self, connect: Connector, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            let connected = cx
                .background_executor()
                .spawn(async move { connect() })
                .await;
            let engine = match connected {
                Ok(engine) => engine,
                Err(error) => {
                    this.update(cx, |this, cx| this.fail(error, cx)).ok();
                    return;
                }
            };
            let info = engine.info().await;
            this.update(cx, |this, cx| match info {
                Ok(info) => {
                    this.status = EngineStatus::Connected(info);
                    this.containers
                        .update(cx, |view, cx| view.attach(engine, cx));
                    cx.notify();
                }
                Err(error) => this.fail(error, cx),
            })
            .ok();
        })
        .detach();
    }

    fn fail(&mut self, error: EngineError, cx: &mut Context<Self>) {
        tracing::warn!(%error, "engine connection failed");
        self.status = EngineStatus::Failed(error.clone());
        self.containers.update(cx, |view, cx| view.fail(error, cx));
        cx.notify();
    }
}

impl Render for AppShell {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .size_full()
            .child(sidebar::render(self.page, &self.status, cx))
            .child(
                div()
                    .flex_1()
                    .h_full()
                    .min_w_0()
                    .child(self.containers.clone()),
            )
    }
}
