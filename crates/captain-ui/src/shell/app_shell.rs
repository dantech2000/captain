use gpui_kit::component::WindowExt;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

mod page_area;

use super::details_rail::Details;
use super::rail::{self, ToggleSidebar};
use super::sidebar;
use super::status_bar::StatusBar;
use crate::containers::ContainersView;
use crate::diagnostics::{DiagnosticsView, diagnostics_model, failures};
use crate::engine_host::{HostEvent, host_model, summary as host_summary};
use crate::extensions::ExtensionsView;
use crate::help;
use crate::images::ImagesView;
use crate::inspector::InspectorView;
use crate::kubernetes::{KubeEvent, kubernetes_model};
use crate::migration::OpenMigrationAssistant;
use crate::networks::NetworksView;
use crate::new_project::{self, NewProject, known_model};
use crate::palette::{CommandPalette, ToggleCommandPalette};
use crate::port_forwarding::PortForwardingView;
use crate::project::{ProjectNotice, ProjectView};
use crate::settings::{self, SettingsView};
use crate::snapshots::SnapshotsView;
use crate::storage::StorageView;
use crate::terminal_panel::{TerminalPanel, ToggleTerminal};
use crate::theme::Palette;
use crate::volumes::VolumesView;
use crate::workspace::{Connection, Connector, Page, Workspace, WorkspaceEvent};

/// The root view: the icon rail, the sidebar unless ⌘B hid it, the current page, the
/// status bar, and the command palette overlay.
pub struct AppShell {
    workspace: Entity<Workspace>,
    containers: Entity<ContainersView>,
    project: Entity<ProjectView>,
    inspector: Entity<InspectorView>,
    images: Entity<ImagesView>,
    volumes: Entity<VolumesView>,
    networks: Entity<NetworksView>,
    extensions: Entity<ExtensionsView>,
    snapshots: Entity<SnapshotsView>,
    storage: Entity<StorageView>,
    forwarding: Entity<PortForwardingView>,
    diagnostics: Entity<DiagnosticsView>,
    settings: Entity<SettingsView>,
    status_bar: Entity<StatusBar>,
    terminal: Entity<TerminalPanel>,
    palette: Option<Entity<CommandPalette>>,
    focus_handle: FocusHandle,
    _subscriptions: Vec<Subscription>,
}

impl AppShell {
    /// A shell with its own workspace, which connects with `connect`.
    pub fn new(connect: Connector, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let workspace = cx.new(|cx| {
            let mut workspace = Workspace::following_settings(cx);
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
        let project = cx.new(|cx| ProjectView::new(workspace.clone(), cx));
        new_project::set_host(&workspace, &project, cx);
        let project_notify = cx.subscribe_in(
            &project,
            window,
            |_, _, notice: &ProjectNotice, window, cx| {
                window.push_notification(notice.notification(), cx);
            },
        );
        let known = known_model(cx);
        let known = cx.observe(&known, |_, _, cx| cx.notify());
        let mut subscriptions = vec![appearance, observe, notify, project_notify, known];
        subscriptions.extend(settings::toast_file_problems(window, cx));

        subscriptions.extend(Self::follow_host(window, cx));
        // The sidebar badge counts failed checks.
        subscriptions
            .extend(diagnostics_model(cx).map(|model| cx.observe(&model, |_, _, cx| cx.notify())));
        let focus_handle = cx.focus_handle();
        focus_handle.focus(window, cx);
        // Keys like ⌘K reach the shell only through focus, so when the focused
        // element goes away the shell takes focus back.
        subscriptions.push(cx.on_focus_lost(window, |this, window, cx| {
            this.focus_handle.focus(window, cx);
        }));

        let terminal = cx.new(|cx| TerminalPanel::new(workspace.clone(), project.clone(), cx));
        Self {
            containers: cx.new(|cx| ContainersView::new(workspace.clone(), cx)),
            project,
            inspector: cx.new(|cx| InspectorView::new(workspace.clone(), cx)),
            images: cx.new(|cx| ImagesView::new(workspace.clone(), cx)),
            volumes: cx.new(|cx| VolumesView::new(workspace.clone(), cx)),
            networks: cx.new(|cx| NetworksView::new(workspace.clone(), cx)),
            extensions: cx.new(|cx| ExtensionsView::new(workspace.clone(), window, cx)),
            snapshots: cx.new(|cx| SnapshotsView::new(workspace.clone(), window, cx)),
            storage: cx.new(|cx| StorageView::new(workspace.clone(), window, cx)),
            forwarding: cx.new(|cx| PortForwardingView::new(workspace.clone(), window, cx)),
            diagnostics: cx.new(|cx| DiagnosticsView::new(workspace.clone(), cx)),
            settings: cx.new(|cx| SettingsView::new(workspace.clone(), cx)),
            status_bar: cx.new(|cx| StatusBar::new(workspace.clone(), cx)),
            terminal,
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
        let mut kubernetes = Vec::new();
        if let Some(model) = kubernetes_model(cx) {
            kubernetes.push(cx.observe(&model, |_, _, cx| cx.notify()));
            kubernetes.push(cx.subscribe_in(
                &model,
                window,
                |_, _, event: &KubeEvent, window, cx| {
                    window.push_notification(event.notification(), cx);
                },
            ));
        }
        let mut subscriptions = vec![
            cx.observe(&host, |_, _, cx| cx.notify()),
            cx.subscribe_in(&host, window, |_, _, event: &HostEvent, window, cx| {
                if let Some(notification) = event.notification() {
                    window.push_notification(notification, cx);
                }
                if *event == HostEvent::OpenMigration {
                    window.dispatch_action(Box::new(OpenMigrationAssistant), cx);
                }
            }),
        ];
        subscriptions.extend(kubernetes);
        subscriptions
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
                if !window.has_active_dialog(cx) {
                    this.focus_handle.focus(window, cx);
                }
                cx.notify();
            })
            .detach();
            palette.focus_handle(cx).focus(window, cx);
            self.palette = Some(palette);
        }
        cx.notify();
    }
}

impl Render for AppShell {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = Palette::of(cx);
        let host = host_summary(cx);
        // The page shows only while the cluster runs. See feature 0024.
        let forwarding =
            kubernetes_model(cx).is_some_and(|model| model.read(cx).status().is_running());
        let workspace = self.workspace.read(cx);
        // The Project page shows the inspector only after a click on a card of its own.
        let show_inspector = workspace.selected().is_some_and(|selected| {
            workspace.page() != Page::Project
                || workspace.card_open()
                    && workspace
                        .focused_group()
                        .is_some_and(|group| group.containers.iter().any(|c| c.id == selected.id))
        }) && !matches!(workspace.connection(), Connection::Failed(_));
        let details = match workspace.selected() {
            Some(container) if show_inspector && workspace.details_hidden() => {
                Details::Hidden(container.display_name().to_string())
            }
            _ if show_inspector => Details::Shown,
            _ => Details::None,
        };

        rail::on_page_actions(div(), &self.workspace, forwarding, self.palette.is_some())
            .size_full()
            .relative()
            .flex()
            .flex_col()
            .bg(palette.bg)
            .text_color(palette.text)
            .text_size(px(13.))
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::toggle_palette))
            .on_action(cx.listener(|this, _: &ToggleSidebar, _, cx| {
                this.workspace
                    .update(cx, |workspace, cx| workspace.toggle_sidebar(cx));
            }))
            .on_action(cx.listener(|this, _: &ToggleTerminal, _, cx| {
                this.workspace
                    .update(cx, |workspace, cx| workspace.toggle_terminal(cx));
            }))
            .on_action(cx.listener(|this, _: &NewProject, window, cx| {
                new_project::open(this.workspace.clone(), this.project.clone(), window, cx);
            }))
            .on_action(cx.listener(|this, _: &OpenMigrationAssistant, window, cx| {
                crate::migration::open(this.workspace.clone(), window, cx);
            }))
            // A click can remove the control under the mouse, so drop its hint.
            .capture_any_mouse_down(|_, _, cx| help::clear(cx))
            // First, so it sees each mouse move before and after every hover listener.
            .child(help::hover_batch())
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .child(rail::render(
                        &self.workspace,
                        workspace,
                        host.as_ref(),
                        failures(cx),
                        forwarding,
                        &palette,
                    ))
                    .when(!workspace.sidebar_hidden(), |row| {
                        row.child(sidebar::render(&self.workspace, workspace, &palette, cx))
                    })
                    .child(self.page_area(
                        workspace.page(),
                        details,
                        (workspace.sidebar_hidden(), workspace.terminal_open()),
                        &palette,
                    )),
            )
            .child(self.status_bar.clone())
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
