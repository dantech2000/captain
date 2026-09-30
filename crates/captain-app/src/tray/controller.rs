//! The menu bar icon: it follows the workspace and rebuilds its menu when the
//! engine state or the container list changes.

use std::collections::HashMap;
use std::time::Duration;

use captain_core::kubernetes::{KubeContexts, load_contexts, user_kubeconfig_paths};
use captain_ui::Workspace;
use gpui_kit::*;
use muda::MenuId;
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

use super::menu::NativeMenu;
use super::menu_model::{self, TrayCommand};
use super::placement::{Rect, popover_rect};
use super::snapshot::{EngineStatus, TraySnapshot};
use super::{events, icon, screens};
use crate::window;

/// Changes often come in bursts, for example `docker compose up`. Wait this long
/// after a change before rebuilding the menu.
const REBUILD_DEBOUNCE: Duration = Duration::from_millis(200);

/// How often the tray reads the kubeconfig for the Kubernetes Contexts submenu.
/// kubectl and other tools change it too.
const CONTEXTS_POLL: Duration = Duration::from_secs(5);

/// Template images on macOS take the menu bar's color, so only the alpha counts.
/// The Windows taskbar is dark by default, so the icon is white there.
const COLOR: [u8; 3] = if cfg!(target_os = "macos") {
    [0, 0, 0]
} else {
    [255, 255, 255]
};

struct Tray {
    icon: TrayIcon,
    /// The snapshot that the current menu shows.
    shown: Option<TraySnapshot>,
    commands: HashMap<MenuId, TrayCommand>,
    rebuild: Option<Task<()>>,
    workspace: Entity<Workspace>,
    contexts: KubeContexts,
    _observe: Vec<Subscription>,
    _events: [Task<()>; 2],
    _contexts: Option<Task<()>>,
}

/// Keeps the tray alive for the life of the app.
struct TrayHandle(Entity<Tray>);

impl Global for TrayHandle {}

/// Puts Captain's icon in the menu bar (macOS) or the notification area (Windows).
/// Call it after the app has launched, on the main thread. If the platform refuses,
/// Captain runs without the icon and quits when its last window closes.
pub fn start(cx: &mut App) {
    let workspace = window::workspace(cx);
    let contexts = load_contexts(&user_kubeconfig_paths());
    let snapshot = snapshot(&workspace, &contexts, cx);
    let host = captain_ui::host_model(cx);
    let icon = TrayIconBuilder::new()
        .with_tooltip("Captain")
        .with_icon(status_icon(snapshot.icon()))
        .with_icon_as_template(true)
        // A left click opens the popover; a right click still opens the menu.
        .with_menu_on_left_click(false)
        .build();
    let icon = match icon {
        Ok(icon) => icon,
        Err(error) => {
            tracing::warn!(%error, "cannot add the menu bar icon");
            return;
        }
    };
    let events = events::listen(command, icon_clicked, cx);
    let tray = cx.new(|cx| {
        let mut observe = vec![cx.observe(&workspace, |tray: &mut Tray, workspace, cx| {
            tray.workspace_changed(workspace, cx);
        })];
        // Captain Engine changes the status line and the Start or Stop item.
        let watched = workspace.clone();
        observe.extend(host.map(|host| {
            cx.observe(&host, move |tray: &mut Tray, _, cx| {
                tray.workspace_changed(watched.clone(), cx);
            })
        }));
        let mut tray = Tray {
            icon,
            shown: None,
            commands: HashMap::new(),
            rebuild: None,
            workspace,
            contexts,
            _observe: observe,
            _events: events,
            _contexts: None,
        };
        tray.show(snapshot);
        tray.poll_contexts(cx);
        tray
    });
    cx.set_global(TrayHandle(tray));
}

/// True while the menu bar icon is up.
pub fn is_running(cx: &App) -> bool {
    cx.has_global::<TrayHandle>()
}

/// Removes the icon. The settings turned it off.
pub fn stop(cx: &mut App) {
    if is_running(cx) {
        captain_ui::close_popover(cx);
        cx.remove_global::<TrayHandle>();
    }
}

/// Opens or closes the popover under the icon. `icon` is in physical pixels.
fn icon_clicked(icon: Rect, cx: &mut App) {
    let screens = screens::screens(cx);
    let place = move |wanted: Size<Pixels>| {
        let relative = cfg!(target_os = "macos");
        let width = wanted.width.as_f32();
        let height = wanted.height.as_f32();
        let (display, rect) = popover_rect(icon, &screens, width, height, relative)?;
        let bounds = Bounds::new(
            point(px(rect.x), px(rect.y)),
            size(px(rect.width), px(rect.height)),
        );
        Some((display, bounds))
    };
    captain_ui::toggle_popover(window::workspace(cx), place, window::show, cx);
}

/// Reads the kubeconfig again now, after the menu switched the context.
pub fn refresh_contexts(cx: &mut App) {
    if let Some(tray) = cx.try_global::<TrayHandle>().map(|handle| handle.0.clone()) {
        tray.update(cx, |tray, cx| {
            tray.set_contexts(load_contexts(&user_kubeconfig_paths()), cx)
        });
    }
}

/// What the item `id` in the current menu does.
fn command(id: &MenuId, cx: &App) -> Option<TrayCommand> {
    let tray = cx.try_global::<TrayHandle>()?;
    tray.0.read(cx).commands.get(id).cloned()
}

impl Tray {
    /// Stats samples notify many times a second. Compare the small snapshot first,
    /// and rebuild only when the menu would change.
    fn workspace_changed(&mut self, workspace: Entity<Workspace>, cx: &mut Context<Self>) {
        let current = snapshot(&workspace, &self.contexts, cx);
        if self.rebuild.is_some() || self.shown.as_ref() == Some(&current) {
            return;
        }
        self.rebuild = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(REBUILD_DEBOUNCE).await;
            this.update(cx, |this, cx| {
                this.rebuild = None;
                this.show(snapshot(&workspace, &this.contexts, cx));
            })
            .ok();
        }));
    }

    /// Reads the contexts every few seconds, off the main thread.
    fn poll_contexts(&mut self, cx: &mut Context<Self>) {
        self._contexts = Some(cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(CONTEXTS_POLL).await;
                let contexts = cx
                    .background_executor()
                    .spawn(async { load_contexts(&user_kubeconfig_paths()) })
                    .await;
                if this
                    .update(cx, |tray, cx| tray.set_contexts(contexts, cx))
                    .is_err()
                {
                    return;
                }
            }
        }));
    }

    fn set_contexts(&mut self, contexts: KubeContexts, cx: &mut Context<Self>) {
        if contexts != self.contexts {
            self.contexts = contexts;
            self.workspace_changed(self.workspace.clone(), cx);
        }
    }

    fn show(&mut self, snapshot: TraySnapshot) {
        if self.shown.as_ref() == Some(&snapshot) {
            return;
        }
        let icon_changed = self.shown.as_ref().map(TraySnapshot::icon) != Some(snapshot.icon());
        if icon_changed
            && let Err(error) = self
                .icon
                .set_icon_with_as_template(Some(status_icon(snapshot.icon())), true)
        {
            tracing::warn!(%error, "cannot update the menu bar icon");
        }
        let native = NativeMenu::new(&menu_model::build(&snapshot));
        self.icon.set_menu(Some(Box::new(native.menu)));
        self.commands = native.commands;
        self.shown = Some(snapshot);
    }
}

fn snapshot(workspace: &Entity<Workspace>, contexts: &KubeContexts, cx: &App) -> TraySnapshot {
    TraySnapshot {
        contexts: contexts.clone(),
        ..TraySnapshot::of(workspace.read(cx), captain_ui::host_summary(cx).as_ref())
    }
}

fn status_icon(status: EngineStatus) -> Icon {
    Icon::from_rgba(icon::rgba(status, COLOR), icon::SIZE, icon::SIZE)
        .expect("the icon buffer matches its size")
}
