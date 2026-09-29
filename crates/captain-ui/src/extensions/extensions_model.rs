use std::sync::Arc;

use captain_core::EngineError;
use captain_core::extension::{
    ExtensionCandidate, ExtensionManager, ExtensionUpdate, InstalledExtension, UpdateCheck,
};
use gpui_kit::*;

use super::ExtensionEvent;
use crate::workspace::Workspace;

/// The installed extensions and the step that runs now: a pull, an install, an
/// update, or a removal. The engine work is in `captain-docker`.
pub struct ExtensionsModel {
    workspace: Entity<Workspace>,
    list: Vec<InstalledExtension>,
    loaded: bool,
    /// What the running step does now, for the page header.
    step: Option<SharedString>,
    task: Option<Task<()>>,
    load: Option<Task<()>>,
    /// The manager the list came from; a new connection reloads.
    source: Option<Arc<dyn ExtensionManager>>,
    _subscription: Subscription,
}

impl EventEmitter<ExtensionEvent> for ExtensionsModel {}

impl ExtensionsModel {
    pub fn new(workspace: Entity<Workspace>, cx: &mut Context<Self>) -> Self {
        let subscription = cx.observe(&workspace, |model: &mut Self, workspace, cx| {
            let manager = workspace.read(cx).extension_manager();
            let same = match (&manager, &model.source) {
                (Some(a), Some(b)) => Arc::ptr_eq(a, b),
                (None, None) => true,
                _ => false,
            };
            if !same {
                if model.source.is_some() {
                    super::close_all_windows(cx);
                }
                model.reload(cx);
            }
        });
        let mut model = Self {
            workspace,
            list: Vec::new(),
            loaded: false,
            step: None,
            task: None,
            load: None,
            source: None,
            _subscription: subscription,
        };
        model.reload(cx);
        model
    }

    pub fn list(&self) -> &[InstalledExtension] {
        &self.list
    }

    pub fn manager(&self, cx: &App) -> Option<Arc<dyn ExtensionManager>> {
        self.workspace.read(cx).extension_manager()
    }

    pub fn is_loaded(&self) -> bool {
        self.loaded
    }

    pub fn is_busy(&self) -> bool {
        self.task.is_some()
    }

    pub fn step(&self) -> Option<SharedString> {
        self.step.clone()
    }

    /// Reads the extensions folder again.
    pub fn reload(&mut self, cx: &mut Context<Self>) {
        let manager = self.manager(cx);
        self.source = manager.clone();
        let Some(manager) = manager else {
            self.list.clear();
            self.loaded = false;
            cx.notify();
            return;
        };
        let list = manager.list();
        self.load = Some(cx.spawn(async move |this, cx| {
            let result = list.await;
            this.update(cx, |model, cx| {
                model.load = None;
                model.loaded = true;
                match result {
                    Ok(list) => model.list = list,
                    Err(error) => cx.emit(ExtensionEvent::Failed {
                        action: "List",
                        message: error.to_string(),
                    }),
                }
                cx.notify();
            })
            .ok();
        }));
    }

    /// Pulls `reference` and checks that it is an extension. The page then asks the
    /// user to confirm.
    pub fn prepare(&mut self, reference: String, cx: &mut Context<Self>) {
        let Some(manager) = self.manager(cx) else {
            return;
        };
        if self.task.is_some() {
            return;
        }
        self.step = Some(format!("Pulling {reference}...").into());
        let prepare = manager.prepare(&reference);
        self.task = Some(cx.spawn(async move |this, cx| {
            let result = prepare.await;
            this.update(cx, |model, cx| {
                model.end_step(cx);
                match result {
                    Ok(candidate) => cx.emit(ExtensionEvent::Confirm(Box::new(candidate))),
                    Err(error) => model.fail("Install", error, cx),
                }
            })
            .ok();
        }));
        cx.notify();
    }

    pub fn install(&mut self, candidate: ExtensionCandidate, cx: &mut Context<Self>) {
        let Some(manager) = self.manager(cx) else {
            return;
        };
        if self.task.is_some() {
            return;
        }
        self.step = Some(format!("Installing {}...", candidate.image).into());
        let install = manager.install(candidate);
        self.task = Some(cx.spawn(async move |this, cx| {
            let result = install.await;
            this.update(cx, |model, cx| {
                model.end_step(cx);
                match result {
                    Ok(extension) => cx.emit(ExtensionEvent::Installed(format!(
                        "Installed {}",
                        extension.title()
                    ))),
                    Err(error) => model.fail("Install", error, cx),
                }
                model.reload(cx);
            })
            .ok();
        }));
        cx.notify();
    }

    /// Pulls the extension's repository with `tag` and compares it with the installed
    /// image. The page then asks the user to confirm the update.
    pub fn check_update(
        &mut self,
        extension: InstalledExtension,
        tag: String,
        cx: &mut Context<Self>,
    ) {
        let Some(manager) = self.manager(cx) else {
            return;
        };
        if self.task.is_some() {
            return;
        }
        let title = extension.title().to_string();
        self.step = Some(format!("Checking {title} for an update...").into());
        let check = manager.check_update(extension, tag);
        self.task = Some(cx.spawn(async move |this, cx| {
            let result = check.await;
            this.update(cx, |model, cx| {
                model.end_step(cx);
                match result {
                    Ok(UpdateCheck::Available(update)) => {
                        cx.emit(ExtensionEvent::ConfirmUpdate(update))
                    }
                    Ok(UpdateCheck::UpToDate { image }) => cx.emit(ExtensionEvent::Done(format!(
                        "{title} is up to date with {image}"
                    ))),
                    Err(error) => model.fail("Update", error, cx),
                }
            })
            .ok();
        }));
        cx.notify();
    }

    /// Closes the extension's window and reinstalls it from the new image.
    pub fn update(&mut self, update: ExtensionUpdate, cx: &mut Context<Self>) {
        let Some(manager) = self.manager(cx) else {
            return;
        };
        if self.task.is_some() {
            return;
        }
        self.step = Some(format!("Updating {}...", update.extension.title()).into());
        super::close_window(&update.extension.id, cx);
        let apply = manager.update(update.extension, update.candidate);
        self.task = Some(cx.spawn(async move |this, cx| {
            let result = apply.await;
            this.update(cx, |model, cx| {
                model.end_step(cx);
                match result {
                    Ok(extension) => cx.emit(ExtensionEvent::Done(format!(
                        "Updated {} to {}",
                        extension.title(),
                        extension.image
                    ))),
                    Err(error) => model.fail("Update", error, cx),
                }
                model.reload(cx);
            })
            .ok();
        }));
        cx.notify();
    }

    pub fn remove(&mut self, extension: InstalledExtension, cx: &mut Context<Self>) {
        let Some(manager) = self.manager(cx) else {
            return;
        };
        if self.task.is_some() {
            return;
        }
        let title = extension.title().to_string();
        self.step = Some(format!("Removing {title}...").into());
        super::close_window(&extension.id, cx);
        let remove = manager.remove(extension);
        self.task = Some(cx.spawn(async move |this, cx| {
            let result = remove.await;
            this.update(cx, |model, cx| {
                model.end_step(cx);
                match result {
                    Ok(()) => cx.emit(ExtensionEvent::Done(format!("Removed {title}"))),
                    Err(error) => model.fail("Remove", error, cx),
                }
                model.reload(cx);
            })
            .ok();
        }));
        cx.notify();
    }

    fn end_step(&mut self, cx: &mut Context<Self>) {
        self.task = None;
        self.step = None;
        cx.notify();
    }

    fn fail(&mut self, action: &'static str, error: EngineError, cx: &mut Context<Self>) {
        let message = match error {
            EngineError::Api(message) | EngineError::Unreachable(message) => message,
        };
        cx.emit(ExtensionEvent::Failed { action, message });
    }
}
