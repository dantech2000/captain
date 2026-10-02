# Feature 0042: App menus

- Milestone: M35
- Status: Built in code (2026-10-01). Not checked by hand in the app yet.
- Design: the standard macOS menu bar, like Zed's

## Goal

Give Captain the menus a Mac app has: Captain, File, Edit, View, Window, and Help. Each item shows the shortcut that its key binding gives, and it is off when its action cannot run.

## Why

- Captain had one menu, Captain, with Quit. Mac users look in the menu bar for commands and shortcuts.
- Without an Edit menu, Copy and Paste have no menu items. AppKit's Services, the window list, and Enter Full Screen need the menus too.

## Scope

### Menus

The order follows Apple's guidelines for the menu bar: the app menu, File, Edit, View, Window, Help.

| Menu | Items |
|------|-------|
| Captain | About Captain · Settings… ⌘, · Services · Hide Captain ⌘H, Hide Others ⌥⌘H, Show All · Quit Captain ⌘Q |
| File | New Project… ⌘N, Open Folder…, New Terminal Tab ⌘T · Close Window ⌘W |
| Edit | Undo ⌘Z, Redo ⇧⌘Z · Cut ⌘X, Copy ⌘C, Paste ⌘V, Select All ⌘A · Find ⌘F |
| View | Command Palette ⌘K, Toggle Sidebar ⌘B, Toggle Terminal ⌃` · Containers ⌘1 … Diagnostics ⌘9 · Enter Full Screen ⌃⌘F (added by AppKit) |
| Window | Minimize ⌘M, Zoom · Bring All to Front · the window list (added by AppKit) |
| Help | Captain Help, Keyboard Shortcuts · Report an Issue…, Show Logs |

- **About Captain** shows the main window and a dialog with the app mark, the versions that the Settings page's About line shows (Captain, the engine runtime, Docker), the license, and links to the licenses and the repository.
- **Settings…** opens the Settings page. While the main window is closed or another Captain window is in front, it opens the main window first. While a dialog or sheet has focus, it closes the dialog first, so the item always does what it says.
- **Open Folder…** opens the New sheet and its Open a folder card at once: the folder dialog, then the sheet's check of the Compose files.
- **New Terminal Tab** opens a tab in the terminal panel, and shows the panel. On a project page the tab opens in the project folder.
- **Captain Help** opens `docs/guide/README.md` on GitHub, **Keyboard Shortcuts** the guide's shortcut table, **Report an Issue…** a new GitHub issue, and **Show Logs** the log folder (as the Diagnostics page's Show logs).
- There is no Check for Updates item. Captain has no updater yet.

### Shortcuts and enabled items

- GPUI shows the first key binding of an item's action next to it (the first one without a key context, else the first). The menus are built as data with the shortcut each item should show, and a test checks it against Captain's key bindings.
- The Edit items send GPUI Kit's text field actions (`input::Undo`, `Copy`, `Search`, and the others). The kit binds ⌘Z, ⌘C, and the rest in its `Input` key context, so the menu shows those keys. Cut, Copy, Paste, and Select All also carry AppKit's selectors, so system panels get them.
- GPUI asks before a menu opens whether each item's action can run (`App::is_action_available`). An action is available when a handler is on the path from the focused element to the window root, or when the app has a global handler. So:
  - The Edit items are on only while a text field, the code editor, or the terminal has focus. Find is on in every text field, but opens a search only in the code editor; other fields pass it on.
  - The page items, New Project, Open Folder, New Terminal Tab, and the View toggles are off while a dialog or sheet has focus (dialogs render beside the shell, not in it), and while no main window is in front.
  - While the ⌘K palette is open, the page items are on, but do nothing, as their keys did before.
  - The Captain, Window, and Help items have global handlers, so they are always on.
- The terminal grid now handles `input::Copy` and `input::Paste`, so Edit > Copy and Paste work there. ⌘C and ⌘V in the grid stay as they were.

### Close Window and ⌘W

- ⌘W is bound to Close Window with no key context. The terminal panel binds ⌘W to Close Tab in its own key context. GPUI prefers the binding of the deeper context, so ⌘W closes a terminal tab while the panel has focus, and the window everywhere else.
- Close Window closes the window in front. Closing the main window keeps Captain running while the menu bar icon is up ([ADR 0006](../adr/0006-menu-bar.md)).
- ⌘T works outside the panel too now: no binding matches there, so AppKit sends the menu item's key equivalent, and the shell opens a tab.

### Platforms

GPUI builds app menus only on macOS. The `app_menu` module and its bindings (⌘H, ⌥⌘H, ⌘W, ⌘M) compile only on macOS. Linux and Windows keep the one Captain menu with Quit.

## Out of scope

- Titles that change with the state (Show Sidebar or Hide Sidebar). GPUI sets the titles once.
- Check for Updates, until Captain has an updater.
- An Open Recent submenu.
- Menus on Linux and Windows.

## Notes

Sources:

- Apple, [The menu bar](https://developer.apple.com/design/human-interface-guidelines/the-menu-bar): the menu order, the app menu items and their order, and "disable the action instead of hiding it".
- GPUI (gpui-pre 0.3.7) `platform/app_menu.rs`: `Menu`, `MenuItem::action`, `separator`, `submenu`, `os_submenu` with `SystemMenuType::Services`, `os_action` with `OsAction`; `init_app_menus` wires validation to `App::is_action_available` and clicks to `App::dispatch_action`. gpui-pre-macos `platform.rs` `create_menu_item` picks the key equivalent from `Keymap::bindings_for_action` with `find_or_first` (see [zed#23621](https://github.com/zed-industries/zed/issues/23621)), and makes the menu named "Window" the window menu.
- Zed's [app_menus.rs](https://github.com/zed-industries/zed/blob/main/crates/zed/src/zed/app_menus.rs): the same item kinds, Services and Hide behind `cfg(target_os = "macos")`, and `os_action` for the Edit items.
- AppKit adds Enter Full Screen to the View menu after launch when no menu item has the `toggleFullScreen:` action ([toggleFullScreen(_:)](https://developer.apple.com/documentation/appkit/nswindow/togglefullscreen(_:)), the `NSFullScreenMenuItemEverywhere` default). GPUI's items use their own selector, so Captain adds no item of its own, which would show twice.

## Verification

Automated:

- `app_menu::entries` test: every menu item with a shortcut shows the keys of a Captain key binding for its action.

By hand, on macOS:

1. Open each menu. The items, separators, and shortcuts match the table above. View ends with Enter Full Screen; Window ends with the window list.
2. Captain > About Captain shows the dialog with the versions; Licenses and Source code open.
3. Close the main window. Captain > Settings… (and ⌘,) opens the window on Settings. About Captain opens the window and the dialog. Open About Captain, then choose Settings…: the dialog closes and Settings shows.
4. Click in a text field. Edit > Undo, Cut, Copy, Paste, and Select All are on and work; ⌘C and ⌘V still work. Click on a page outside fields: the Edit items are off.
5. Open a Compose file on a project's Files tab. Edit > Find opens the editor's search; ⌘F does too.
6. Select text in the terminal panel. Edit > Copy copies it; Edit > Paste pastes into the shell.
7. Open the New sheet (⌘N). The View page items and File > New Project are off; ⌘1 still picks the first card.
8. Focus the terminal panel and press ⌘W: a tab closes. Click on the page and press ⌘W: the window closes and the menu bar icon stays.
9. File > Open Folder… shows the folder dialog over the New sheet. Pick a folder with a Compose file: the project opens.
10. Window > Minimize, Zoom, and Bring All to Front (with a floating log window open) work. Captain > Hide Captain, Hide Others, and Show All work.
11. Help > Captain Help, Keyboard Shortcuts, and Report an Issue… open GitHub. Show Logs opens the log folder.
