use captain_core::migration::{MigrationRun, StepStatus, TransferEvent};
use futures::StreamExt;
use gpui_kit::*;

use super::assistant::{MigrationAssistant, Stage};

impl MigrationAssistant {
    /// Starts copying the selected items. Items that an earlier run in this window
    /// already copied or skipped are not copied again.
    pub(super) fn start(&mut self, cx: &mut Context<Self>) {
        let Some(plan) = &self.plan else {
            return;
        };
        let mut run = MigrationRun::new(plan);
        run.resume_from(&self.run);
        self.run = run;
        self.stage = Stage::Run;
        self.error = None;
        self.run_next(cx);
    }

    /// Copies the next pending item, or shows the summary when none is left.
    fn run_next(&mut self, cx: &mut Context<Self>) {
        let Some(session) = self.session.clone() else {
            return;
        };
        let Some(ix) = self.run.next_pending() else {
            self.task = None;
            self.stage = Stage::Summary;
            // Remove helpers now; the session stays open for retries.
            cx.background_executor().spawn(session.finish()).detach();
            cx.notify();
            return;
        };
        let entry = &self.run.entries[ix];
        let total = entry.item.size();
        let mut events = session.copy(&entry.item, entry.snapshot);
        self.run
            .set_status(ix, StepStatus::Running { done: 0, total });
        cx.notify();
        self.task = Some(cx.spawn(async move |this, cx| {
            let mut result = StepStatus::Done;
            while let Some(event) = events.next().await {
                let status = match event {
                    Ok(TransferEvent::Progress { done, total }) => {
                        StepStatus::Running { done, total }
                    }
                    Ok(TransferEvent::Note(note)) => {
                        this.update(cx, |this, _| this.run.set_note(ix, note)).ok();
                        continue;
                    }
                    Ok(TransferEvent::Skipped(reason)) => {
                        result = StepStatus::Skipped(reason);
                        continue;
                    }
                    Err(error) => {
                        result = StepStatus::Failed(error.to_string());
                        break;
                    }
                };
                this.update(cx, |this, cx| {
                    this.run.set_status(ix, status);
                    cx.notify();
                })
                .ok();
            }
            this.update(cx, |this, cx| {
                this.run.set_status(ix, result);
                this.run_next(cx);
            })
            .ok();
        }));
    }

    /// Stops the run. The item that is copying stops too: its helpers and any
    /// half-copied volume are removed, and it goes back in the queue.
    pub(super) fn stop(&mut self, cx: &mut Context<Self>) {
        self.task = None;
        self.run.stop();
        cx.notify();
    }

    /// Goes on with the items that are not done.
    pub(super) fn resume(&mut self, cx: &mut Context<Self>) {
        if self.task.is_none() {
            self.stage = Stage::Run;
            self.run_next(cx);
        }
    }

    /// Copies a failed item again.
    pub(super) fn retry(&mut self, ix: usize, cx: &mut Context<Self>) {
        if self.run.retry(ix) {
            self.resume(cx);
            cx.notify();
        }
    }

    /// Copies every failed item again.
    pub(super) fn retry_failed(&mut self, cx: &mut Context<Self>) {
        for ix in 0..self.run.entries.len() {
            self.run.retry(ix);
        }
        self.resume(cx);
    }

    /// True while an item is copying.
    pub(super) fn is_copying(&self) -> bool {
        self.task.is_some() && self.stage == Stage::Run
    }
}
