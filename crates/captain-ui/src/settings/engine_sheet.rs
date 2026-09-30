//! The engines sheet, from "Add a remote host…" in the engine menu: the endpoint,
//! the engines on this computer, the Docker contexts, and a remote host field.

use gpui_kit::component::WindowExt;
use gpui_kit::*;

use super::{SettingsView, endpoint_picker, store};
use crate::theme::Palette;
use crate::widgets::primary_button;

pub struct EngineSheet {
    view: Entity<SettingsView>,
    _observe: Subscription,
}

/// Looks for engines again and opens the sheet.
pub fn open(view: Entity<SettingsView>, window: &mut Window, cx: &mut App) {
    view.update(cx, |view, cx| view.rescan(cx));
    let sheet = cx.new(|cx| EngineSheet {
        _observe: cx.observe(&view, |_, _, cx| cx.notify()),
        view,
    });
    window.open_dialog(cx, move |dialog, _, _| {
        dialog
            .title("Engines and remote hosts")
            .w(px(620.))
            .margin_top(px(110.))
            .child(sheet.clone())
    });
}

impl Render for EngineSheet {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = Palette::of(cx);
        let settings = store::current(cx);
        let this = self.view.downgrade();
        let view = self.view.read(cx);
        let picker =
            endpoint_picker::render(view, &this, &settings, view.input.as_ref(), &palette, cx);
        div()
            .flex()
            .flex_col()
            .gap(px(14.))
            .child(
                div()
                    .id("engine-sheet-scroll")
                    .max_h(px(520.))
                    .overflow_y_scroll()
                    .child(picker),
            )
            .child(div().flex().justify_end().child(primary_button(
                "engine-sheet-done",
                "Done",
                "Close the engines sheet.",
                true,
                &palette,
                |_, window, cx| window.close_dialog(cx),
            )))
    }
}
