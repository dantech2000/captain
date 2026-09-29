use captain_core::model::ContainerAction;
use gpui_kit::*;

use super::{Workspace, WorkspaceEvent};

impl Workspace {
    /// Runs `action` on a container. The event stream reloads the list when the
    /// engine finishes, so this only tracks the pending state. It emits a
    /// [`WorkspaceEvent`] when the action fails or deletes the container.
    pub fn run_action(&mut self, id: String, action: ContainerAction, cx: &mut Context<Self>) {
        let Some(engine) = self.engine.clone() else {
            return;
        };
        if !self.pending.insert(id.clone()) {
            return;
        }
        let name = self
            .store
            .find(&id)
            .map_or_else(|| id.clone(), |c| c.name.clone());
        cx.notify();
        cx.spawn(async move |this, cx| {
            let result = engine.run_action(&id, action).await;
            this.update(cx, |this, cx| {
                this.pending.remove(&id);
                match result {
                    Ok(()) if action.removes() => {
                        cx.emit(WorkspaceEvent::ContainerRemoved { name })
                    }
                    Ok(()) => {}
                    Err(error) => {
                        tracing::warn!(%error, ?action, "container action failed");
                        cx.emit(WorkspaceEvent::ActionFailed {
                            id,
                            name,
                            action,
                            error,
                        });
                    }
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    /// Runs `action` on each of `ids`, for example every container of a project.
    pub fn run_actions(
        &mut self,
        ids: Vec<String>,
        action: ContainerAction,
        cx: &mut Context<Self>,
    ) {
        for id in ids {
            self.run_action(id, action, cx);
        }
    }

    /// True while an action on this container is running.
    pub fn is_pending(&self, id: &str) -> bool {
        self.pending.contains(id)
    }
}
