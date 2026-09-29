//! Selecting several containers and acting on them at once. See
//! docs/features/0018-bulk-selection.md.

use captain_core::model::{BulkOutcome, Container, ContainerAction};
use captain_core::store::{MultiSelection, SelectMode};
use futures::future::join_all;
use gpui_kit::*;

use super::{Workspace, WorkspaceEvent};

impl Workspace {
    /// The containers the user selected for a bulk action.
    pub fn bulk(&self) -> &MultiSelection {
        &self.checked
    }

    /// The selected containers that still exist, in the order the user added them.
    pub fn bulk_containers(&self) -> Vec<Container> {
        self.checked
            .keys()
            .iter()
            .filter_map(|id| self.store.find(id).cloned())
            .collect()
    }

    /// A click on a row, with the keys held: selects it alone, adds or removes it,
    /// or selects a range. The inspector shows the clicked row, unless the click
    /// removed it from the selection.
    pub fn click_row(&mut self, id: String, mode: SelectMode, cx: &mut Context<Self>) {
        let order = self.row_order();
        self.checked.click(&id, mode, &order);
        if self.checked.contains(&id) {
            self.selected = Some(id);
        } else if self.selected.as_deref() == Some(id.as_str()) {
            self.selected = self.checked.keys().last().cloned();
        }
        cx.notify();
    }

    pub fn clear_bulk(&mut self, cx: &mut Context<Self>) {
        let selected = self.selected.clone();
        self.checked.clear();
        if let Some(id) = selected {
            self.checked.click(&id, SelectMode::Replace, &[]);
        }
        cx.notify();
    }

    /// Runs each container's action, then emits one [`WorkspaceEvent::BulkDone`].
    /// `verb` names the action in the result, for example Delete for a mix of
    /// Delete and Stop and delete.
    pub fn run_bulk(
        &mut self,
        targets: Vec<(Container, ContainerAction)>,
        verb: ContainerAction,
        cx: &mut Context<Self>,
    ) {
        let Some(engine) = self.engine.clone() else {
            return;
        };
        let targets: Vec<_> = targets
            .into_iter()
            .filter(|(c, _)| self.pending.insert(c.id.clone()))
            .collect();
        if targets.is_empty() {
            return;
        }
        cx.notify();
        let runs = targets.into_iter().map(|(container, action)| {
            let run = engine.run_action(&container.id, action);
            async move { (container, run.await) }
        });
        let runs = join_all(runs);
        cx.spawn(async move |this, cx| {
            let results = runs.await;
            this.update(cx, |this, cx| {
                let mut outcome = BulkOutcome::default();
                for (container, result) in results {
                    this.pending.remove(&container.id);
                    match result {
                        Ok(()) => outcome.done.push(container.name),
                        Err(error) => {
                            tracing::warn!(%error, ?verb, "bulk container action failed");
                            outcome.failed.push((container.name, error));
                        }
                    }
                }
                cx.emit(WorkspaceEvent::BulkDone {
                    action: verb,
                    outcome,
                });
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    /// Every row the list shows, in order. Rows in a folded card do not count.
    fn row_order(&self) -> Vec<String> {
        self.visible_groups()
            .into_iter()
            .filter(|group| !self.is_collapsed(&group.project))
            .flat_map(|group| group.containers.into_iter().map(|c| c.id))
            .collect()
    }
}
