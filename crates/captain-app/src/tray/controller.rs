//! The menu bar icon: it follows the workspace and rebuilds its native menu when
//! the engine state, the container list, or a problem changes. A left click opens
//! the menu, like a right click.

use std::collections::HashMap;
use std::time::Duration;

use captain_core::kubernetes::{KubeContexts, load_contexts, user_kubeconfig_paths};
use captain_core::model::ContainerState;
use captain_ui::Workspace;
use gpui_kit::*;
use muda::{MenuId, MenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

use super::exit_facts::ExitFactsCache;
use super::gather::snapshot;
use super::menu::NativeMenu;
use super::menu_model::{self, TrayCommand};
use super::snapshot::{EngineStatus, TraySnapshot};
use super::{events, icon};
use crate::window;

/// Changes often come in bursts, for example `docker compose up`. Wait this long
/// after a change before rebuilding the menu.
const REBUILD_DEBOUNCE: Duration = Duration::from_millis(200);

/// How long each frame of the turning wheel shows while the engine starts.
const TURN_FRAME: Duration = Duration::from_millis(120);

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

pub(super) struct Tray {
    icon: TrayIcon,
    /// The snapshot that the current menu shows.
    shown: Option<TraySnapshot>,
    commands: HashMap<MenuId, TrayCommand>,
    /// The engine's status line in the current menu.
    status: Option<MenuItem>,
    rebuild: Option<Task<()>>,
    workspace: Entity<Workspace>,
    contexts: KubeContexts,
    pub(super) exits: ExitFactsCache,
    _observe: Vec<Subscription>,
    _events: Task<()>,
    _contexts: Option<Task<()>>,
    /// Turns the wheel while the engine starts.
    spin: Option<Task<()>>,
}

/// Keeps the tray alive for the life of the app.
pub(super) struct TrayHandle(pub(super) Entity<Tray>);

impl Global for TrayHandle {}

/// Puts Captain's icon in the menu bar (macOS) or the notification area (Windows).
/// Call it after the app has launched, on the main thread. If the platform refuses,
/// Captain runs without the icon and quits when its last window closes.
pub fn start(cx: &mut App) {
    let workspace = window::workspace(cx);
    let contexts = load_contexts(&user_kubeconfig_paths());
    let snapshot = snapshot(&workspace, &contexts, &HashMap::new(), cx);
    let host = captain_ui::host_model(cx);
    // tray-icon opens the menu on a left click by default, as macOS menu extras do.
    let icon = TrayIconBuilder::new()
        .with_tooltip("Captain")
        .with_icon(status_icon(snapshot.icon(), 0))
        .with_icon_as_template(true)
        .build();
    let icon = match icon {
        Ok(icon) => icon,
        Err(error) => {
            tracing::warn!(%error, "cannot add the menu bar icon");
            return;
        }
    };
    let events = events::listen(command, cx);
    let tray = cx.new(|cx| {
        let mut observe = vec![cx.observe(&workspace, |tray: &mut Tray, _, cx| {
            tray.follow_restarts(cx);
            tray.workspace_changed(cx);
        })];
        // Captain Engine changes the status line and the Start or Stop item; the
        // checks and Kubernetes change the problem and the Kubernetes items.
        observe.extend(
            host.map(|m| cx.observe(&m, |tray: &mut Tray, _, cx| tray.workspace_changed(cx))),
        );
        observe.extend(
            captain_ui::diagnostics_model(cx)
                .map(|m| cx.observe(&m, |tray: &mut Tray, _, cx| tray.workspace_changed(cx))),
        );
        observe.extend(
            captain_ui::kubernetes_model(cx)
                .map(|m| cx.observe(&m, |tray: &mut Tray, _, cx| tray.workspace_changed(cx))),
        );
        let mut tray = Tray {
            icon,
            shown: None,
            commands: HashMap::new(),
            status: None,
            rebuild: None,
            workspace,
            contexts,
            exits: ExitFactsCache::default(),
            _observe: observe,
            _events: events,
            _contexts: None,
            spin: None,
        };
        tray.show(snapshot, cx);
        tray.follow_restarts(cx);
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
        cx.remove_global::<TrayHandle>();
    }
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
    pub(super) fn workspace_changed(&mut self, cx: &mut Context<Self>) {
        let current = snapshot(&self.workspace, &self.contexts, &self.exits.facts(), cx);
        if self.rebuild.is_some() || self.shown.as_ref() == Some(&current) {
            return;
        }
        self.rebuild = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(REBUILD_DEBOUNCE).await;
            this.update(cx, |this, cx| {
                this.rebuild = None;
                let current = snapshot(&this.workspace, &this.contexts, &this.exits.facts(), cx);
                this.show(current, cx);
            })
            .ok();
        }));
    }

    /// Inspects each container that crashed again since the last look, to learn if
    /// it ran out of memory and what its limit is.
    fn follow_restarts(&mut self, cx: &mut Context<Self>) {
        let workspace = self.workspace.read(cx);
        let crashing = workspace
            .store()
            .containers()
            .iter()
            .filter_map(|c| {
                let crash = workspace.recent_crash(&c.id);
                (c.state == ContainerState::Restarting || crash.is_some())
                    .then(|| (c.id.clone(), crash.map(|crash| crash.at)))
            })
            .collect();
        let engine = workspace.engine();
        let fresh = self.exits.follow(crashing);
        let Some(engine) = engine else {
            return;
        };
        for id in fresh {
            let inspect = engine.inspect_container(&id);
            cx.spawn(async move |this, cx| {
                let Ok(detail) = inspect.await else {
                    return;
                };
                this.update(cx, |tray, cx| {
                    if tray.exits.insert(id, detail) {
                        tray.workspace_changed(cx);
                    }
                })
                .ok();
            })
            .detach();
        }
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
            self.workspace_changed(cx);
        }
    }

    /// Shows `snapshot`. When only the status line changed, as it does with each
    /// stats sample, the line changes in place; else the menu is built again.
    fn show(&mut self, snapshot: TraySnapshot, cx: &mut Context<Self>) {
        let shown = self.shown.as_ref();
        if shown == Some(&snapshot) {
            return;
        }
        if let (Some(shown), Some(status)) = (shown, &self.status)
            && shown.same_menu(&snapshot)
        {
            status.set_text(menu_model::status_text(&snapshot));
            self.shown = Some(snapshot);
            return;
        }
        let status = snapshot.icon();
        if shown.map(TraySnapshot::icon) != Some(status) {
            self.set_icon(status, 0);
            self.spin = (status == EngineStatus::Starting).then(|| Self::turn(cx));
        }
        let native = NativeMenu::new(&menu_model::build(&snapshot));
        self.icon.set_menu(Some(Box::new(native.menu)));
        self.commands = native.commands;
        self.status = native.status;
        self.shown = Some(snapshot);
    }
}

impl Tray {
    fn set_icon(&self, status: EngineStatus, frame: u32) {
        if let Err(error) = self
            .icon
            .set_icon_with_as_template(Some(status_icon(status, frame)), true)
        {
            tracing::warn!(%error, "cannot update the menu bar icon");
        }
    }

    /// Advances the wheel one frame at a time until the task is dropped.
    fn turn(cx: &mut Context<Self>) -> Task<()> {
        cx.spawn(async move |this, cx| {
            let mut frame = 0;
            loop {
                cx.background_executor().timer(TURN_FRAME).await;
                frame = (frame + 1) % icon::TURN_FRAMES;
                if this
                    .update(cx, |tray, _| tray.set_icon(EngineStatus::Starting, frame))
                    .is_err()
                {
                    return;
                }
            }
        })
    }
}

fn status_icon(status: EngineStatus, frame: u32) -> Icon {
    Icon::from_rgba(icon::rgba(status, frame, COLOR), icon::SIZE, icon::SIZE)
        .expect("the icon buffer matches its size")
}
