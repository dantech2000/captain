use captain_core::migration::RollbackStatus;
use gpui_kit::*;

use super::assistant::MigrationAssistant;

impl MigrationAssistant {
    /// Rolls back the switch-over of run item `ix`: stops it here and starts the
    /// original in the old engine again. Nothing is removed on either side.
    pub(super) fn roll_back(&mut self, ix: usize, cx: &mut Context<Self>) {
        let Some(session) = self.session.clone() else {
            return;
        };
        let Some(entry) = self.run.entries.get(ix) else {
            return;
        };
        if !entry
            .switch_over
            .as_ref()
            .is_some_and(|progress| progress.can_roll_back())
        {
            return;
        }
        let rolled_back = session.roll_back(&entry.item);
        self.run.set_rollback(ix, RollbackStatus::Running);
        cx.notify();
        cx.spawn(async move |this, cx| {
            let status = match rolled_back.await {
                Ok(()) => RollbackStatus::Done,
                Err(error) => RollbackStatus::Failed(error.to_string()),
            };
            this.update(cx, |this, cx| {
                this.run.set_rollback(ix, status);
                cx.notify();
            })
            .ok();
            // The session stays alive until the roll back ends.
            drop(session);
        })
        .detach();
    }
}
