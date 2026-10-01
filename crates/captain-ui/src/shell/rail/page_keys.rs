//! Keyboard shortcuts for the rail's pages: ⌘1 to ⌘9 in rail order, and ⌘, for
//! Settings, as in other Mac apps. Ctrl takes the place of ⌘ on Linux and Windows.

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
    vec![
        KeyBinding::new(&key(Page::Containers), ShowContainers, None),
        KeyBinding::new(&key(Page::Images), ShowImages, None),
        KeyBinding::new(&key(Page::Volumes), ShowVolumes, None),
        KeyBinding::new(&key(Page::Networks), ShowNetworks, None),
        KeyBinding::new(&key(Page::Snapshots), ShowSnapshots, None),
        KeyBinding::new(&key(Page::Storage), ShowStorage, None),
        KeyBinding::new(&key(Page::Extensions), ShowExtensions, None),
        KeyBinding::new(&key(Page::PortForwarding), ShowPortForwarding, None),
        KeyBinding::new(&key(Page::Diagnostics), ShowDiagnostics, None),
        KeyBinding::new(&key(Page::Settings), ShowSettings, None),
    ]
}

/// Adds a handler for each page action to `root`, which switches the page.
/// Port Forwarding opens only while `forwarding`, as its rail button shows only then.
pub fn on_page_actions(root: Div, handle: &Entity<Workspace>, forwarding: bool) -> Div {
    let go = |page: Page| {
        let handle = handle.clone();
        move |cx: &mut App| handle.update(cx, |workspace, cx| workspace.set_page(page, cx))
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
    root.on_action(move |_: &ShowContainers, _, cx| c(cx))
        .on_action(move |_: &ShowImages, _, cx| i(cx))
        .on_action(move |_: &ShowVolumes, _, cx| v(cx))
        .on_action(move |_: &ShowNetworks, _, cx| n(cx))
        .on_action(move |_: &ShowSnapshots, _, cx| sn(cx))
        .on_action(move |_: &ShowStorage, _, cx| st(cx))
        .on_action(move |_: &ShowExtensions, _, cx| e(cx))
        .on_action(move |_: &ShowPortForwarding, _, cx| {
            if forwarding {
                pf(cx)
            }
        })
        .on_action(move |_: &ShowDiagnostics, _, cx| d(cx))
        .on_action(move |_: &ShowSettings, _, cx| se(cx))
}

#[cfg(test)]
mod tests;
