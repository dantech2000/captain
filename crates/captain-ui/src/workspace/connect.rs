use std::sync::Arc;
use std::time::Duration;

use captain_core::{Engine, EngineError};
use gpui_kit::*;

use super::{Connection, Workspace};

/// Connects to an engine. It may block, so the workspace runs it on a background thread.
pub type Connector = Box<dyn FnOnce() -> Result<Arc<dyn Engine>, EngineError> + Send>;

impl Workspace {
    /// Connects in the background, then loads containers and follows events.
    pub fn connect(&mut self, connect: Connector, cx: &mut Context<Self>) {
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
                    this.connection = Connection::Connected(info);
                    this.engine = Some(engine);
                    this.reload(Duration::ZERO, cx);
                    this.watch_events(cx);
                    cx.notify();
                }
                Err(error) => this.fail(error, cx),
            })
            .ok();
        })
        .detach();
    }
}
