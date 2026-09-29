use gpui_kit::component::WindowExt;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use super::sidebar;
use crate::containers::ContainersView;
use crate::engine_host::{HostEvent, host_model, summary as host_summary};
use crate::images::ImagesView;
use crate::inspector::InspectorView;
use crate::migration::OpenMigrationAssistant;
use crate::networks::NetworksView;
use crate::palette::{CommandPalette, ToggleCommandPalette};
use crate::settings::{self, SettingsView};
use crate::theme::Palette;
use crate::volumes::VolumesView;
use crate::workspace::{Connection, Connector, Page, Workspace, WorkspaceEvent};

/// The root view: sidebar, the current page, and the command palette overlay.
pub struct AppShell {
    workspace: Entity<Workspace>,
    containers: Entity<ContainersView>,
    inspector: Entity<InspectorView>,
    images: Entity<ImagesView>,
    volumes: Entity<VolumesView>,
    networks: Entity<NetworksView>,
    settings: Entity<SettingsView>,
    palette: Option<Entity<CommandPalette>>,
    focus_handle: FocusHandle,
    _subscriptions: Vec<Subscription>,
}

impl AppShell {
    /// A shell with its own workspace, which connects with `connect`.
    pub fn new(connect: Connector, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let workspace = cx.new(|cx| {
            let mut workspace = Workspace::new();
            workspace.connect(connect, cx);
            workspace
        });
        Self::with_workspace(workspace, window, cx)
    }

    /// A shell for a workspace that someone else owns, so the workspace outlives
    /// the window. The menu bar icon uses this: closing the window keeps the engine
    /// connection.
    pub fn with_workspace(
        workspace: Entity<Workspace>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        // The appearance setting can force light or dark; `System` follows the window.
        settings::apply_appearance(Some(window), cx);
        let appearance = window
            .observe_window_appearance(|window, cx| settings::apply_appearance(Some(window), cx));

        let observe = cx.observe(&workspace, |_, _, cx| cx.notify());
        // Action failures and deletions become toasts in the window's notification layer.
        let notify = cx.subscribe_in(
            &workspace,
            window,
            |_, _, event: &WorkspaceEvent, window, cx| {
                window.push_notification(event.notification(), cx);
            },
        );
        let mut subscriptions = vec![appearance, observe, notify];
        subscriptions.extend(Self::follow_host(window, cx));
        let focus_handle = cx.focus_handle();
        focus_handle.focus(window, cx);

        Self {
            containers: cx.new(|cx| ContainersView::new(workspace.clone(), cx)),
            inspector: cx.new(|cx| InspectorView::new(workspace.clone(), cx)),
            images: cx.new(|cx| ImagesView::new(workspace.clone(), cx)),
            volumes: cx.new(|cx| VolumesView::new(workspace.clone(), cx)),
            networks: cx.new(|cx| NetworksView::new(workspace.clone(), cx)),
            settings: cx.new(|cx| SettingsView::new(workspace.clone(), cx)),
            workspace,
            palette: None,
            focus_handle,
            _subscriptions: subscriptions,
        }
    }

    /// Redraws when Captain Engine changes, shows its failures as toasts, and opens
    /// the Migration Assistant after a setup that asked for it.
    fn follow_host(window: &mut Window, cx: &mut Context<Self>) -> Vec<Subscription> {
        let Some(host) = host_model(cx) else {
            return Vec::new();
        };
        vec![
            cx.observe(&host, |_, _, cx| cx.notify()),
            cx.subscribe_in(&host, window, |_, _, event: &HostEvent, window, cx| {
                if let Some(notification) = event.notification() {
                    window.push_notification(notification, cx);
                }
                if *event == HostEvent::OpenMigration {
                    window.dispatch_action(Box::new(OpenMigrationAssistant), cx);
                }
            }),
        ]
    }

    fn toggle_palette(
        &mut self,
        _: &ToggleCommandPalette,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.palette.take().is_some() {
            self.focus_handle.focus(window, cx);
        } else {
            let palette = cx.new(|cx| CommandPalette::new(self.workspace.clone(), window, cx));
            cx.subscribe_in(&palette, window, |this, _, _: &DismissEvent, window, cx| {
                this.palette = None;
                this.focus_handle.focus(window, cx);
                cx.notify();
            })
            .detach();
            palette.focus_handle(cx).focus(window, cx);
            self.palette = Some(palette);
        }
        cx.notify();
    }

    fn page(&self, page: Page, show_inspector: bool) -> Div {
        let main = div().flex_1().min_w_0().h_full();
        let row = div().flex_1().min_w_0().h_full().flex();
        match page {
            Page::Containers => row
                .child(main.child(self.containers.clone()))
                .when(show_inspector, |this| this.child(self.inspector.clone())),
            Page::Images => row.child(main.child(self.images.clone())),
            Page::Volumes => row.child(main.child(self.volumes.clone())),
            Page::Networks => row.child(main.child(self.networks.clone())),
            Page::Settings => row.child(main.child(self.settings.clone())),
        }
    }
}

impl Render for AppShell {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = Palette::of(cx);
        let host = host_summary(cx);
        let workspace = self.workspace.read(cx);
        let show_inspector = workspace.selected().is_some()
            && !matches!(workspace.connection(), Connection::Failed(_));

        div()
            .size_full()
            .relative()
            .flex()
            .bg(palette.bg)
            .text_color(palette.text)
            .text_size(px(13.))
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::toggle_palette))
            .on_action(cx.listener(|this, _: &OpenMigrationAssistant, window, cx| {
                crate::migration::open(this.workspace.clone(), window, cx);
            }))
            .child(sidebar::render(
                &self.workspace,
                workspace,
                host.as_ref(),
                &palette,
            ))
            .child(self.page(workspace.page(), show_inspector))
            .children(self.palette.clone().map(|command_palette| {
                div()
                    .absolute()
                    .inset_0()
                    .bg(hsla(0., 0., 0., 0.45))
                    .flex()
                    .justify_center()
                    .items_start()
                    .pt(px(140.))
                    .child(command_palette)
            }))
    }
}
