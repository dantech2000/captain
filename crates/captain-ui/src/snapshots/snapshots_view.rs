use captain_core::format::bytes_label;
use gpui_kit::component::WindowExt;
use gpui_kit::component::spinner::Spinner;
use gpui_kit::*;

use super::{SnapshotEvent, SnapshotsModel, create_dialog, snapshot_row};
use crate::engine_host::host_model;
use crate::help::HelpExt;
use crate::icons::CaptainIcon;
use crate::theme::Palette;
use crate::widgets::{
    ButtonTone, empty_note, inline_error, page_header, settings_card, text_button,
};
use crate::workspace::{Page, Workspace};

/// The Snapshots page: the list, Create, and each row's Restore and Delete.
pub struct SnapshotsView {
    model: Entity<SnapshotsModel>,
    /// True while the page shows, to read the folder again when it opens.
    showing: bool,
    _subscriptions: Vec<Subscription>,
}

impl SnapshotsView {
    pub fn new(workspace: Entity<Workspace>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let model = cx.new(SnapshotsModel::new);
        let mut subscriptions = vec![
            cx.observe(&model, |_, _, cx| cx.notify()),
            cx.subscribe_in(&model, window, |_, _, event: &SnapshotEvent, window, cx| {
                window.push_notification(event.notification(), cx);
            }),
            // The CLI can change the folder too, so read it each time the page opens.
            cx.observe(&workspace, |view: &mut Self, workspace, cx| {
                let showing = workspace.read(cx).page() == Page::Snapshots;
                if showing && !view.showing {
                    view.model.update(cx, |model, cx| model.reload(cx));
                }
                view.showing = showing;
            }),
        ];
        subscriptions.extend(host_model(cx).map(|host| cx.observe(&host, |_, _, cx| cx.notify())));
        Self {
            model,
            showing: false,
            _subscriptions: subscriptions,
        }
    }
}

/// "3 snapshots · 120 GB free", or the step that runs now.
fn summary(model: &SnapshotsModel) -> SharedString {
    if let Some(step) = model.step() {
        return step;
    }
    let count = model.list().snapshots.len();
    let noun = if count == 1 { "snapshot" } else { "snapshots" };
    match model.list().free_bytes {
        Some(free) => format!("{count} {noun} · {} free", bytes_label(free)).into(),
        None => format!("{count} {noun}").into(),
    }
}

impl Render for SnapshotsView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = Palette::of(cx);
        let handle = self.model.clone();
        let model = self.model.read(cx);
        let (running, engine_busy) = host_model(cx).map_or((false, false), |host| {
            let host = host.read(cx);
            let busy = host.status().is_busy() || host.is_snapshotting();
            (host.status().is_running(), busy)
        });
        let enabled = model.is_available() && !model.is_busy() && !engine_busy;
        let create = text_button(
            "snapshots-create",
            "Create snapshot…",
            ButtonTone::Accent,
            enabled,
            &palette,
            move |_, window, cx| create_dialog::open(handle.clone(), running, window, cx),
        )
        .help("Save the state of Captain Engine: its disk, images, containers, and volumes.")
        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation());
        let trailing = div()
            .flex()
            .items_center()
            .gap(px(10.))
            .children(model.is_busy().then(|| Spinner::new().color(palette.text2)))
            .child(create);

        let body: AnyElement = if !model.is_available() {
            inline_error("Snapshots need Captain Engine on macOS.", &palette).into_any_element()
        } else if model.is_loaded() && model.list().snapshots.is_empty() {
            empty_note(
                CaptainIcon::Snapshot,
                "No snapshots",
                "Save the engine's state to go back to it later.",
                &palette,
            )
            .into_any_element()
        } else {
            let rows = model
                .list()
                .snapshots
                .iter()
                .map(|snapshot| snapshot_row::render(&self.model, snapshot, enabled, &palette));
            settings_card("Snapshots", rows, &palette).into_any_element()
        };

        div()
            .size_full()
            .flex()
            .flex_col()
            .child(page_header(
                "snapshots-header",
                "Snapshots",
                summary(model),
                Some(trailing.into_any_element()),
                &palette,
            ))
            .child(
                div()
                    .id("snapshots-scroll")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .px(px(24.))
                    .pt(px(4.))
                    .pb(px(24.))
                    .child(div().w_full().max_w(px(720.)).child(body)),
            )
    }
}
