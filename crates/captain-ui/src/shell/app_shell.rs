use gpui_kit::component::Theme;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use super::sidebar;
use crate::containers::ContainersView;
use crate::inspector::InspectorView;
use crate::theme::Palette;
use crate::workspace::{Connection, Connector, Workspace};

/// The root view: sidebar, container list, and inspector.
pub struct AppShell {
    workspace: Entity<Workspace>,
    containers: Entity<ContainersView>,
    inspector: Entity<InspectorView>,
    _subscriptions: Vec<Subscription>,
}

impl AppShell {
    pub fn new(connect: Connector, window: &mut Window, cx: &mut Context<Self>) -> Self {
        Theme::sync_system_appearance(Some(window), cx);
        let appearance = window.observe_window_appearance(|window, cx| {
            Theme::sync_system_appearance(Some(window), cx)
        });

        let workspace = cx.new(|cx| {
            let mut workspace = Workspace::new();
            workspace.connect(connect, cx);
            workspace
        });
        let containers = cx.new(|cx| ContainersView::new(workspace.clone(), cx));
        let inspector = cx.new(|cx| InspectorView::new(workspace.clone(), cx));
        let observe = cx.observe(&workspace, |_, _, cx| cx.notify());

        Self {
            workspace,
            containers,
            inspector,
            _subscriptions: vec![appearance, observe],
        }
    }
}

impl Render for AppShell {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = Palette::of(cx);
        let workspace = self.workspace.read(cx);
        let show_inspector = workspace.selected().is_some()
            && !matches!(workspace.connection(), Connection::Failed(_));

        div()
            .size_full()
            .flex()
            .bg(palette.bg)
            .text_color(palette.text)
            .text_size(px(13.))
            .child(sidebar::render(workspace, &palette))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .child(self.containers.clone()),
            )
            .when(show_inspector, |this| this.child(self.inspector.clone()))
    }
}
