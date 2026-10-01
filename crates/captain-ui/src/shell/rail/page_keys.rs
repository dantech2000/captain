//! Keyboard shortcuts for the rail's pages: ⌘1 to ⌘9 in rail order, and ⌘, for
//! Settings, as in other Mac apps. Ctrl takes the place of ⌘ on Linux and Windows.
//! They apply only in the shell's key context, and do nothing while a dialog,
//! a sheet, or the ⌘K palette is open, so those keep ⌘1 and the others.

use gpui_kit::component::WindowExt;
use gpui_kit::*;

use crate::workspace::{Page, Workspace};

gpui_kit::actions!(
    captain,
    [
        ShowContainers,
        ShowImages,
        ShowVolumes,
        ShowNetworks,
        ShowSnapshots,
        ShowStorage,
        ShowExtensions,
        ShowPortForwarding,
        ShowDiagnostics,
        ShowSettings
    ]
);

/// The key context of the shell's root. The kit's dialogs render beside the
/// shell, not in it, so the page keys never match while a dialog has focus.
pub const SHELL_CONTEXT: &str = "CaptainShell";

/// The key that goes with ⌘ (or Ctrl) for `page`, if it has one.
pub fn page_key(page: Page) -> Option<&'static str> {
    Some(match page {
        Page::Containers => "1",
        Page::Images => "2",
        Page::Volumes => "3",
        Page::Networks => "4",
        Page::Snapshots => "5",
        Page::Storage => "6",
        Page::Extensions => "7",
        Page::PortForwarding => "8",
        Page::Diagnostics => "9",
        Page::Settings => ",",
        Page::Project => return None,
    })
}

/// The key bindings, for the app to register: `cmd-` or `ctrl-` and each page key.
pub fn page_bindings(modifier: &str) -> Vec<KeyBinding> {
    let key = |page| format!("{modifier}-{}", page_key(page).unwrap_or_default());
    let context = Some(SHELL_CONTEXT);
    vec![
        KeyBinding::new(&key(Page::Containers), ShowContainers, context),
        KeyBinding::new(&key(Page::Images), ShowImages, context),
        KeyBinding::new(&key(Page::Volumes), ShowVolumes, context),
        KeyBinding::new(&key(Page::Networks), ShowNetworks, context),
        KeyBinding::new(&key(Page::Snapshots), ShowSnapshots, context),
        KeyBinding::new(&key(Page::Storage), ShowStorage, context),
        KeyBinding::new(&key(Page::Extensions), ShowExtensions, context),
        KeyBinding::new(&key(Page::PortForwarding), ShowPortForwarding, context),
        KeyBinding::new(&key(Page::Diagnostics), ShowDiagnostics, context),
        KeyBinding::new(&key(Page::Settings), ShowSettings, context),
    ]
}

/// Adds a handler for each page action to `root`, which switches the page.
/// Port Forwarding opens only while `forwarding`, as its rail button shows only then.
/// Nothing switches while `palette_open`, or while a dialog or sheet is open.
pub fn on_page_actions(
    root: Div,
    handle: &Entity<Workspace>,
    forwarding: bool,
    palette_open: bool,
) -> Div {
    let go = |page: Page| {
        let handle = handle.clone();
        move |window: &mut Window, cx: &mut App| {
            if palette_open || window.has_active_dialog(cx) {
                cx.propagate();
                return;
            }
            handle.update(cx, |workspace, cx| workspace.set_page(page, cx))
        }
    };
    let (c, i, v, n, sn, st, e, d, se) = (
        go(Page::Containers),
        go(Page::Images),
        go(Page::Volumes),
        go(Page::Networks),
        go(Page::Snapshots),
        go(Page::Storage),
        go(Page::Extensions),
        go(Page::Diagnostics),
        go(Page::Settings),
    );
    let pf = go(Page::PortForwarding);
    root.key_context(SHELL_CONTEXT)
        .on_action(move |_: &ShowContainers, w, cx| c(w, cx))
        .on_action(move |_: &ShowImages, w, cx| i(w, cx))
        .on_action(move |_: &ShowVolumes, w, cx| v(w, cx))
        .on_action(move |_: &ShowNetworks, w, cx| n(w, cx))
        .on_action(move |_: &ShowSnapshots, w, cx| sn(w, cx))
        .on_action(move |_: &ShowStorage, w, cx| st(w, cx))
        .on_action(move |_: &ShowExtensions, w, cx| e(w, cx))
        .on_action(move |_: &ShowPortForwarding, w, cx| {
            if forwarding {
                pf(w, cx)
            }
        })
        .on_action(move |_: &ShowDiagnostics, w, cx| d(w, cx))
        .on_action(move |_: &ShowSettings, w, cx| se(w, cx))
}

#[cfg(test)]
mod tests;
