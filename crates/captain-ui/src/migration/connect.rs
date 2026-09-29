use captain_core::migration::{ImageChoice, MigrationRun};
use gpui_kit::*;

use super::assistant::{MigrationAssistant, Stage, finish};

impl MigrationAssistant {
    /// Connects to `host` and the target in the background, then reads the plan and
    /// the target's free space.
    pub(super) fn choose(&mut self, host: String, cx: &mut Context<Self>) {
        let (Some(backend), Some(target)) = (self.backend.clone(), self.target.clone()) else {
            return;
        };
        if host.trim().is_empty() {
            self.error = Some("Enter an endpoint, for example unix:///path/to/docker.sock.".into());
            cx.notify();
            return;
        }
        self.close_session(cx);
        self.stage = Stage::Loading;
        self.error = None;
        cx.notify();
        self.task = Some(cx.spawn(async move |this, cx| {
            let opened = cx
                .background_executor()
                .spawn(async move { backend.open(host.trim(), &target) })
                .await;
            let session = match opened {
                Ok(session) => session,
                Err(error) => {
                    this.update(cx, |this, cx| {
                        this.fail(error.to_string(), Stage::Choose, cx)
                    })
                    .ok();
                    return;
                }
            };
            let (plan, free) = futures::join!(session.scan(), session.target_free_space());
            this.update(cx, |this, cx| {
                this.session = Some(session);
                match plan {
                    Ok(plan) => {
                        this.plan = Some(plan);
                        this.free = free.ok().flatten();
                        this.run = MigrationRun::default();
                        this.stage = Stage::Review;
                        this.task = None;
                        cx.notify();
                    }
                    Err(error) => this.fail(error.to_string(), Stage::Choose, cx),
                }
            })
            .ok();
        }));
    }

    /// Uses the endpoint typed into the custom field.
    pub(super) fn choose_custom(&mut self, cx: &mut Context<Self>) {
        let host = self.custom.read(cx).value().to_string();
        self.choose(host, cx);
    }

    /// Goes back to the engine list and closes the session.
    pub(super) fn back_to_choose(&mut self, cx: &mut Context<Self>) {
        self.task = None;
        self.close_session(cx);
        self.plan = None;
        self.error = None;
        self.stage = Stage::Choose;
        cx.notify();
    }

    /// Goes back from the run to the plan, to change the selection and go on.
    pub(super) fn back_to_review(&mut self, cx: &mut Context<Self>) {
        self.stop(cx);
        self.stage = Stage::Review;
        cx.notify();
    }

    pub(super) fn toggle(&mut self, key: &str, cx: &mut Context<Self>) {
        if let Some(plan) = &mut self.plan {
            plan.toggle(key);
            cx.notify();
        }
    }

    pub(super) fn set_snapshot(&mut self, key: &str, snapshot: bool, cx: &mut Context<Self>) {
        if let Some(plan) = &mut self.plan {
            plan.set_snapshot(key, snapshot);
            cx.notify();
        }
    }

    pub(super) fn set_image_choice(&mut self, choice: ImageChoice, cx: &mut Context<Self>) {
        if let Some(plan) = &mut self.plan {
            plan.set_image_choice(choice);
            cx.notify();
        }
    }

    /// Removes the session's helpers in the background and drops it.
    fn close_session(&mut self, cx: &mut Context<Self>) {
        if let Some(session) = self.session.take() {
            finish(session, cx);
        }
    }
}
