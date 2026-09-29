//! The window side of the bridge: routes each page message to the engine or answers
//! it here, and sends the replies back into the page.

use captain_core::extension::{BridgeEvent, BridgeRequest, Route, parse_call};
use futures::StreamExt;
use gpui_kit::*;
use serde_json::{Value, json};

use super::extension_window::ExtensionWindow;

impl ExtensionWindow {
    /// Handles one message from `window.ipc.postMessage`. The extension comes from
    /// this window, never from the message.
    pub(super) fn handle(&mut self, message: &str, window: &mut Window, cx: &mut Context<Self>) {
        let call = match parse_call(message) {
            Ok(call) => call,
            Err(error) => {
                tracing::debug!(message = %error.message, "extension bridge call refused");
                if let Some(id) = error.id {
                    self.deliver(id, &BridgeEvent::error(error.message), cx);
                }
                return;
            }
        };
        let id = call.id;
        match call.request.route() {
            Route::Engine => {
                let mut events = self.manager.call(&self.extension, call.request);
                let task = cx.spawn(async move |this, cx| {
                    while let Some(event) = events.next().await {
                        let last = event.is_last();
                        let delivered = this.update(cx, |this, cx| this.deliver(id, &event, cx));
                        if delivered.is_err() || last {
                            break;
                        }
                    }
                    this.update(cx, |this, _| this.calls.remove(&id)).ok();
                });
                self.calls.insert(id, task);
            }
            Route::Window => self.answer(id, call.request, window, cx),
        }
    }

    /// Toasts, the open panel, links, and closing a stream.
    fn answer(
        &mut self,
        id: u64,
        request: BridgeRequest,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let done = BridgeEvent::Resolve(Value::Null);
        match request {
            BridgeRequest::Toast { level, message } => {
                self.show_toast(level, message, cx);
                self.deliver(id, &done, cx);
            }
            BridgeRequest::OpenExternal(url) => {
                let event = if self.open_external(&url, cx) {
                    done
                } else {
                    BridgeEvent::error(format!("Captain opens only web links, not {url}"))
                };
                self.deliver(id, &event, cx);
            }
            BridgeRequest::Close(target) => {
                // Dropping the task drops the stream, which stops the command.
                self.calls.remove(&target);
                self.deliver(id, &done, cx);
            }
            BridgeRequest::OpenDialog(options) => {
                let paths = cx.prompt_for_paths(PathPromptOptions {
                    files: options.files,
                    directories: options.directories,
                    multiple: options.multiple,
                    prompt: None,
                });
                cx.spawn_in(window, async move |this, cx| {
                    let chosen = match paths.await {
                        Ok(Ok(Some(paths))) => paths,
                        _ => Vec::new(),
                    };
                    let paths: Vec<String> =
                        chosen.iter().map(|p| p.display().to_string()).collect();
                    let result = json!({ "canceled": paths.is_empty(), "filePaths": paths });
                    this.update(cx, |this, cx| {
                        this.deliver(id, &BridgeEvent::Resolve(result), cx)
                    })
                    .ok();
                })
                .detach();
            }
            engine => {
                let event = BridgeEvent::error(format!("{engine:?} is not a window call"));
                self.deliver(id, &event, cx);
            }
        }
    }

    /// Runs the reply script in the page.
    pub(super) fn deliver(&mut self, id: u64, event: &BridgeEvent, cx: &mut Context<Self>) {
        let Some(webview) = &self.webview else {
            return;
        };
        if let Err(error) = webview.read(cx).raw().evaluate_script(&event.script(id)) {
            tracing::warn!(%error, "cannot answer the extension page");
        }
    }
}
