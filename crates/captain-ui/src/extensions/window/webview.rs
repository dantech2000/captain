//! Builds the native web view: the UI files on a custom scheme, the `ddClient` init
//! script, the IPC handler, and a navigation policy that keeps the page on its own
//! scheme and sends other links to the browser.

use captain_core::extension::{
    InstalledExtension, SCHEME, ShimContext, data_store_id, init_script,
};
use futures::channel::mpsc::UnboundedSender;
use gpui_kit::Window;
use raw_window_handle::HasWindowHandle;
use wry::{NewWindowResponse, WebViewBuilder, WebViewBuilderExtDarwin};

use super::protocol;

/// What the web view hands to the window view.
pub enum Inbound {
    /// A message from `window.ipc.postMessage`.
    Message(String),
    /// A link that leaves the extension, for the system browser.
    External(String),
}

/// The web view for `extension`, as a child of `window`. It sends page messages and
/// outside links to `tx`.
pub fn build(
    extension: &InstalledExtension,
    ui_dir: std::path::PathBuf,
    hostname: String,
    tx: UnboundedSender<Inbound>,
    window: &mut Window,
) -> Result<wry::WebView, String> {
    let id = extension.id.clone();
    let url = extension
        .page_url()
        .ok_or("The extension's page is not on this computer, so Captain does not open it.")?;
    // The files on `captain-ext://<id>/`, or the backend's page on localhost.
    let own = format!("{}/", extension.page_origin().unwrap_or_default());
    let context = ShimContext::for_host(id.clone(), extension.image.clone(), hostname);
    let (messages, links, windows) = (tx.clone(), tx.clone(), tx);
    let builder = WebViewBuilder::new()
        .with_initialization_script(init_script(&context))
        .with_custom_protocol(SCHEME.into(), move |_, request| {
            protocol::serve(&ui_dir, &id, &request)
        })
        .with_ipc_handler(move |request| {
            messages
                .unbounded_send(Inbound::Message(request.into_body()))
                .ok();
        })
        .with_navigation_handler(move |url| {
            let allowed = url.starts_with(&own)
                || url == own.trim_end_matches('/')
                || url.starts_with("about:");
            if !allowed {
                links.unbounded_send(Inbound::External(url)).ok();
            }
            allowed
        })
        .with_new_window_req_handler(move |url, _| {
            windows.unbounded_send(Inbound::External(url)).ok();
            NewWindowResponse::Deny
        })
        .with_devtools(cfg!(debug_assertions))
        .with_data_store_identifier(data_store_id(&extension.id))
        .with_url(url);
    let handle = window
        .window_handle()
        .map_err(|error| format!("The window has no native handle: {error}"))?;
    builder
        .build_as_child(&handle)
        .map_err(|error| format!("Captain cannot create the web view: {error}"))
}
