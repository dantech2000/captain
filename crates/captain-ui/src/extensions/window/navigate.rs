//! `ddClient.desktopUI.navigate`: brings Captain's main window forward and opens
//! the page, the selection, or the inspector tab the call names. The engine finds a
//! named container, image, or volume first, so the promise fails for one that does
//! not exist, as the SDK says.

use captain_core::extension::{BridgeEvent, BridgeRequest, NavigateIntent};
use futures::StreamExt;
use gpui_kit::*;
use serde_json::Value;

use super::extension_window::ExtensionWindow;

impl ExtensionWindow {
    pub(super) fn navigate(&mut self, id: u64, intent: NavigateIntent, cx: &mut Context<Self>) {
        if !intent.names_object() {
            let event = self.show(&intent, String::new(), cx);
            self.deliver(id, &event, cx);
            return;
        }
        let mut found = self
            .manager
            .call(&self.extension, BridgeRequest::Navigate(intent.clone()));
        let task = cx.spawn(async move |this, cx| {
            let answer = found.next().await;
            this.update(cx, |this, cx| {
                let event = match answer {
                    Some(BridgeEvent::Resolve(Value::String(full))) => this.show(&intent, full, cx),
                    Some(other @ BridgeEvent::Reject(_)) => other,
                    _ => BridgeEvent::error("Captain could not find it"),
                };
                this.deliver(id, &event, cx);
                this.calls.remove(&id);
            })
            .ok();
        });
        self.calls.insert(id, task);
    }

    /// Opens the page for `intent` in the main window. `found` is the full ID or
    /// name the engine answered.
    fn show(
        &mut self,
        intent: &NavigateIntent,
        found: String,
        cx: &mut Context<Self>,
    ) -> BridgeEvent {
        let main = self.main.clone();
        let raised = main
            .window
            .update(cx, |_, window, _| window.activate_window())
            .is_ok();
        if !raised {
            return BridgeEvent::error("Captain's main window is closed. Open it and try again.");
        }
        main.workspace
            .update(cx, |workspace, cx| workspace.navigate(intent, found, cx));
        cx.activate(true);
        BridgeEvent::Resolve(Value::Null)
    }
}
