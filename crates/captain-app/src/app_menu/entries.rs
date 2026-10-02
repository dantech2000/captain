//! The menus as data: each item's title, action, and the shortcut it should show.
//! GPUI shows an action's first key binding next to its item, so the shortcuts
//! here only document what the bindings give, and a test compares the two.

use captain_ui::{
    NewProject, NewTerminalTab, OpenProjectFolder, ShowContainers, ShowDiagnostics, ShowExtensions,
    ShowImages, ShowNetworks, ShowPortForwarding, ShowSettings, ShowSnapshots, ShowStorage,
    ShowVolumes, ToggleCommandPalette, ToggleSidebar, ToggleTerminal,
};
use gpui_kit::component::input::{Copy, Cut, Paste, Redo, Search, SelectAll, Undo};
use gpui_kit::*;

use crate::actions::Quit;

gpui_kit::actions!(
    captain,
    [
        About,
        Hide,
        HideOthers,
        ShowAll,
        CloseWindow,
        Minimize,
        Zoom,
        BringAllToFront,
        OpenGuide,
        OpenShortcuts,
        ReportIssue,
        ShowLogs
    ]
);

/// One line of a menu.
pub enum Entry {
    Item {
        name: &'static str,
        action: Box<dyn Action>,
        os_action: Option<OsAction>,
        /// The shortcut the item shows, in key binding syntax. `None` for items
        /// without one, and for the Edit items, whose keys GPUI Kit's text fields bind.
        /// Only the test reads it.
        #[cfg_attr(not(test), expect(dead_code))]
        keys: Option<&'static str>,
    },
    Separator,
    Services,
}

pub struct MenuSpec {
    pub name: &'static str,
    pub entries: Vec<Entry>,
}

fn item(name: &'static str, action: impl Action, keys: Option<&'static str>) -> Entry {
    Entry::Item {
        name,
        action: Box::new(action),
        os_action: None,
        keys,
    }
}

/// An Edit item that macOS knows by its selector, so system panels get it too.
fn edit(name: &'static str, action: impl Action, os_action: OsAction) -> Entry {
    Entry::Item {
        name,
        action: Box::new(action),
        os_action: Some(os_action),
        keys: None,
    }
}

/// Captain, File, Edit, View, Window, and Help, in the order Apple's guidelines
/// give. AppKit adds Enter Full Screen to View, the window list to Window, and
/// the search field to Help.
pub fn menu_specs() -> Vec<MenuSpec> {
    use Entry::Separator;
    let spec = |name, entries| MenuSpec { name, entries };
    vec![
        spec(
            "Captain",
            vec![
                item("About Captain", About, None),
                Separator,
                item("Settings…", ShowSettings, Some("cmd-,")),
                Separator,
                Entry::Services,
                Separator,
                item("Hide Captain", Hide, Some("cmd-h")),
                item("Hide Others", HideOthers, Some("alt-cmd-h")),
                item("Show All", ShowAll, None),
                Separator,
                item("Quit Captain", Quit, Some("cmd-q")),
            ],
        ),
        spec(
            "File",
            vec![
                item("New Project…", NewProject, Some("cmd-n")),
                item("Open Folder…", OpenProjectFolder, None),
                item("New Terminal Tab", NewTerminalTab, Some("cmd-t")),
                Separator,
                item("Close Window", CloseWindow, Some("cmd-w")),
            ],
        ),
        spec(
            "Edit",
            vec![
                edit("Undo", Undo, OsAction::Undo),
                edit("Redo", Redo, OsAction::Redo),
                Separator,
                edit("Cut", Cut, OsAction::Cut),
                edit("Copy", Copy, OsAction::Copy),
                edit("Paste", Paste, OsAction::Paste),
                edit("Select All", SelectAll, OsAction::SelectAll),
                Separator,
                item("Find", Search, None),
            ],
        ),
        spec(
            "View",
            vec![
                item("Command Palette", ToggleCommandPalette, Some("cmd-k")),
                item("Toggle Sidebar", ToggleSidebar, Some("cmd-b")),
                item("Toggle Terminal", ToggleTerminal, Some("ctrl-`")),
                Separator,
                item("Containers", ShowContainers, Some("cmd-1")),
                item("Images", ShowImages, Some("cmd-2")),
                item("Volumes", ShowVolumes, Some("cmd-3")),
                item("Networks", ShowNetworks, Some("cmd-4")),
                item("Snapshots", ShowSnapshots, Some("cmd-5")),
                item("Storage", ShowStorage, Some("cmd-6")),
                item("Extensions", ShowExtensions, Some("cmd-7")),
                item("Port Forwarding", ShowPortForwarding, Some("cmd-8")),
                item("Diagnostics", ShowDiagnostics, Some("cmd-9")),
            ],
        ),
        spec(
            "Window",
            vec![
                item("Minimize", Minimize, Some("cmd-m")),
                item("Zoom", Zoom, None),
                Separator,
                item("Bring All to Front", BringAllToFront, None),
            ],
        ),
        spec(
            "Help",
            vec![
                item("Captain Help", OpenGuide, None),
                item("Keyboard Shortcuts", OpenShortcuts, None),
                Separator,
                item("Report an Issue…", ReportIssue, None),
                item("Show Logs", ShowLogs, None),
            ],
        ),
    ]
}

/// The GPUI menus for `specs`.
pub fn menus(specs: Vec<MenuSpec>) -> Vec<Menu> {
    specs
        .into_iter()
        .map(|spec| {
            Menu::new(spec.name).items(spec.entries.into_iter().map(|entry| match entry {
                Entry::Item {
                    name,
                    action,
                    os_action,
                    ..
                } => MenuItem::Action {
                    name: name.into(),
                    action,
                    os_action,
                    checked: false,
                    disabled: false,
                },
                Entry::Separator => MenuItem::separator(),
                Entry::Services => MenuItem::os_submenu("Services", SystemMenuType::Services),
            }))
        })
        .collect()
}

/// The keys of the items that only the menu has. ⌘W closes the window, but the
/// terminal panel's ⌘W binding is in a deeper key context, so it wins and closes
/// a tab while the panel has focus.
pub fn menu_bindings() -> Vec<KeyBinding> {
    vec![
        KeyBinding::new("cmd-h", Hide, None),
        KeyBinding::new("alt-cmd-h", HideOthers, None),
        KeyBinding::new("cmd-w", CloseWindow, None),
        KeyBinding::new("cmd-m", Minimize, None),
    ]
}

#[cfg(test)]
mod tests;
