//! The extension window's view: a slim GPUI title bar, a toast strip for
//! `desktopUI.toast`, and the web view below. GPUI overlays cannot draw above a
//! native web view, so toasts take their own strip. See ADR 0011.

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use captain_core::extension::{ExtensionManager, InstalledExtension, ToastLevel};
use futures::StreamExt;
use futures::channel::mpsc;
use gpui_kit::assets::IconName;
use gpui_kit::component::Icon;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use gpui_wry::WebView;

use super::webview::{self, Inbound};
use crate::theme::Palette;
use crate::widgets::{drag_region, inline_error};

/// How long a success or warning toast stays. Errors stay until closed.
const TOAST_TIME: Duration = Duration::from_secs(5);

pub struct ExtensionWindow {
    pub(super) extension: InstalledExtension,
    pub(super) manager: Arc<dyn ExtensionManager>,
    pub(super) webview: Option<Entity<WebView>>,
    error: Option<String>,
    toast: Option<(ToastLevel, String)>,
    toast_timer: Option<Task<()>>,
    /// The engine calls that are still answering, by call ID. Dropping one stops it.
    pub(super) calls: HashMap<u64, Task<()>>,
    _inbound: Option<Task<()>>,
}

impl ExtensionWindow {
    pub fn new(
        extension: InstalledExtension,
        manager: Arc<dyn ExtensionManager>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        // The title bar follows the appearance setting, like the main window.
        crate::settings::apply_appearance(Some(window), cx);
        let (tx, mut rx) = mpsc::unbounded();
        let ui_dir = manager.paths().ui_dir(&extension.id);
        let built = webview::build(&extension, ui_dir, hostname(), tx, window);
        let (webview, error, inbound) = match built {
            Ok(native) => {
                let view = cx.new(|cx| WebView::new(native, window, cx));
                let inbound = cx.spawn_in(window, async move |this, cx| {
                    while let Some(inbound) = rx.next().await {
                        let handled = this.update_in(cx, |this, window, cx| match inbound {
                            Inbound::Message(message) => this.handle(&message, window, cx),
                            Inbound::External(url) => {
                                this.open_external(&url, cx);
                            }
                        });
                        if handled.is_err() {
                            break;
                        }
                    }
                });
                (Some(view), None, Some(inbound))
            }
            Err(error) => {
                tracing::warn!(%error, id = %extension.id, "cannot open the extension page");
                (None, Some(error), None)
            }
        };
        Self {
            extension,
            manager,
            webview,
            error,
            toast: None,
            toast_timer: None,
            calls: HashMap::new(),
            _inbound: inbound,
        }
    }

    /// Opens `url` in the system browser, for web links only.
    pub(super) fn open_external(&mut self, url: &str, cx: &mut Context<Self>) -> bool {
        let web = url.starts_with("https://") || url.starts_with("http://");
        if web {
            cx.open_url(url);
        } else {
            tracing::warn!(%url, "an extension tried to open a link that is not a web page");
        }
        web
    }

    pub(super) fn show_toast(
        &mut self,
        level: ToastLevel,
        message: String,
        cx: &mut Context<Self>,
    ) {
        self.toast = Some((level, message));
        self.toast_timer = (level != ToastLevel::Error).then(|| {
            cx.spawn(async move |this, cx| {
                cx.background_executor().timer(TOAST_TIME).await;
                this.update(cx, |this, cx| this.hide_toast(cx)).ok();
            })
        });
        cx.notify();
    }

    fn hide_toast(&mut self, cx: &mut Context<Self>) {
        self.toast = None;
        self.toast_timer = None;
        cx.notify();
    }

    fn toast_strip(&self, palette: &Palette, cx: &mut Context<Self>) -> Option<Div> {
        let (level, message) = self.toast.clone()?;
        let (icon, color) = match level {
            ToastLevel::Success => (IconName::CircleCheck, palette.green),
            ToastLevel::Warning => (IconName::TriangleAlert, palette.orange),
            ToastLevel::Error => (IconName::CircleX, palette.red),
        };
        Some(
            div()
                .flex_shrink_0()
                .px(px(14.))
                .py(px(8.))
                .flex()
                .items_center()
                .gap(px(8.))
                .bg(palette.tint(color))
                .border_b_1()
                .border_color(palette.sep)
                .child(Icon::new(icon).size(px(14.)).text_color(color))
                .child(div().flex_1().min_w_0().text_size(px(12.)).child(message))
                .child(
                    div()
                        .id("extension-toast-close")
                        .cursor_pointer()
                        .child(
                            Icon::new(IconName::X)
                                .size(px(14.))
                                .text_color(palette.text2),
                        )
                        .on_click(cx.listener(|this, _, _, cx| this.hide_toast(cx))),
                ),
        )
    }
}

/// The computer's name, for `ddClient.host.hostname`.
fn hostname() -> String {
    static NAME: OnceLock<String> = OnceLock::new();
    NAME.get_or_init(|| {
        std::process::Command::new("/bin/hostname")
            .output()
            .ok()
            .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
            .unwrap_or_default()
    })
    .clone()
}

impl Render for ExtensionWindow {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = Palette::of(cx);
        let title = self.extension.title().to_string();
        let toast = self.toast_strip(&palette, cx);
        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(palette.bg)
            .text_color(palette.text)
            .text_size(px(13.))
            .child(
                drag_region("extension-title")
                    .h(px(38.))
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .border_b_1()
                    .border_color(palette.sep)
                    .bg(palette.side)
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(title),
            )
            .children(toast)
            .when_some(self.error.clone(), |this, error| {
                this.child(div().p(px(24.)).child(inline_error(error, &palette)))
            })
            .children(
                self.webview
                    .clone()
                    .map(|webview| div().flex_1().min_h_0().child(webview)),
            )
    }
}
