use captain_core::model::ContainerAction;
use gpui_kit::*;

use super::Workspace;

impl Workspace {
    /// Runs `action` on a container. The event stream reloads the list when the
    /// engine finishes, so this only tracks the pending state and any error.
    pub fn run_action(&mut self, id: String, action: ContainerAction, cx: &mut Context<Self>) {
        let Some(engine) = self.engine.clone() else {
            return;
        };
        if !self.pending.insert(id.clone()) {
            return;
        }
        self.action_error = None;
        cx.notify();
        cx.spawn(async move |this, cx| {
            let result = engine.run_action(&id, action).await;
            this.update(cx, |this, cx| {
                this.pending.remove(&id);
                if let Err(error) = result {
                    tracing::warn!(%error, ?action, "container action failed");
                    this.action_error = Some(format!("{} failed: {error}", action.label()));
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    /// True while an action on this container is running.
    pub fn is_pending(&self, id: &str) -> bool {
        self.pending.contains(id)
    }

    /// The error from the last action, if it failed.
    pub fn action_error(&self) -> Option<&str> {
        self.action_error.as_deref()
    }
}
