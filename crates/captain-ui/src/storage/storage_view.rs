use captain_core::storage::largest_items;
use gpui_kit::component::WindowExt;
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::component::spinner::Spinner;
use gpui_kit::*;

use super::{StorageEvent, StorageModel, disk_header, largest_list, reclaim_panel, storage_model};
use crate::engine_host::{host_model, uses_captain};
use crate::icons::CaptainIcon;
use crate::settings;
use crate::theme::Palette;
use crate::widgets::{empty_note, inline_error, page_header};
use crate::workspace::Workspace;

/// The most rows "Largest first" lists.
const LARGEST: usize = 12;

/// The Storage page. The app's [`StorageModel`] holds the data.
pub struct StorageView {
    workspace: Entity<Workspace>,
    model: Option<Entity<StorageModel>>,
    _subscriptions: Vec<Subscription>,
}

impl StorageView {
    pub fn new(workspace: Entity<Workspace>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let model = storage_model(cx);
        let mut subscriptions = Vec::new();
        if let Some(model) = &model {
            subscriptions.push(cx.observe(model, |_, _, cx| cx.notify()));
            subscriptions.push(cx.subscribe_in(
                model,
                window,
                |_, _, event: &StorageEvent, window, cx| {
                    window.push_notification(event.notification(), cx);
                },
            ));
        }
        subscriptions.extend(host_model(cx).map(|host| cx.observe(&host, |_, _, cx| cx.notify())));
        Self {
            workspace,
            model,
            _subscriptions: subscriptions,
        }
    }
}

impl Render for StorageView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = Palette::of(cx);
        let Some(handle) = self.model.clone() else {
            return div().size_full();
        };
        let model = handle.read(cx);
        let page = div().size_full().flex().flex_col();
        let Some(usage) = model.usage() else {
            let busy = model.step().or_else(|| {
                model
                    .is_loading()
                    .then(|| SharedString::from("Reading the disk use..."))
            });
            let body = match (&model.error, busy) {
                (_, Some(step)) => div()
                    .px(px(24.))
                    .flex()
                    .gap(px(8.))
                    .text_color(palette.text2)
                    .child(Spinner::new().color(palette.text2))
                    .child(step),
                (Some(error), None) => inline_error(error.clone(), &palette).px(px(24.)),
                (None, None) => empty_note(
                    CaptainIcon::Reclaim,
                    "No engine",
                    "Connect to an engine to see its disk use.",
                    &palette,
                ),
            };
            let header = page_header(
                "storage-header",
                "Storage",
                "What fills the engine's disk",
                None,
                &palette,
            );
            return page.child(header).child(body);
        };
        let captain = uses_captain(cx);
        let can_snapshot = model.can_snapshot(cx);
        let weekly = settings::current(cx).weekly_build_cache_cleanup;
        let items = largest_items(usage, model.plan(), &model.snapshots, LARGEST);
        let header = model.breakdown(cx).map(|breakdown| {
            disk_header::render(
                &breakdown,
                captain,
                model.host_free,
                &self.workspace,
                &palette,
            )
        });
        page.children(header)
            .children(
                model
                    .error
                    .clone()
                    .map(|error| inline_error(error, &palette).px(px(24.)).pt(px(12.))),
            )
            .child(
                div()
                    .id("storage-scroll")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scrollbar()
                    .child(
                        div()
                            .flex()
                            .items_start()
                            .gap(px(24.))
                            .px(px(24.))
                            .py(px(20.))
                            .child(largest_list::render(&items, &palette))
                            .child(reclaim_panel::render(
                                &handle,
                                model,
                                can_snapshot,
                                weekly,
                                &palette,
                            )),
                    ),
            )
    }
}
