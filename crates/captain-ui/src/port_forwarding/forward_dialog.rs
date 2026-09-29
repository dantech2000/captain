//! The Forward dialog: a local port above 1024, or empty for a free one.

use captain_core::kubernetes::{ForwardKey, check_local_port};
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::{Sizable, WindowExt};
use gpui_kit::*;

use super::ForwardingModel;
use crate::images::field;
use crate::theme::Palette;
use crate::widgets::{ButtonTone, inline_error, text_button};

struct ForwardDialog {
    model: Entity<ForwardingModel>,
    key: ForwardKey,
    port: Entity<InputState>,
    error: Option<String>,
    _subscription: Subscription,
}

impl ForwardDialog {
    fn new(
        model: Entity<ForwardingModel>,
        key: ForwardKey,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let port = cx.new(|cx| InputState::new(window, cx).placeholder("Any free port"));
        let subscription = cx.subscribe_in(&port, window, |this, _, event, window, cx| {
            if let InputEvent::PressEnter { .. } = event {
                this.submit(window, cx);
            }
        });
        Self {
            model,
            key,
            port,
            error: None,
            _subscription: subscription,
        }
    }

    fn submit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let text = self.port.read(cx).value().trim().to_string();
        let port = if text.is_empty() {
            None
        } else {
            match text
                .parse::<u16>()
                .map_err(|_| "Enter a port number.".to_string())
            {
                Ok(port) => match check_local_port(port) {
                    Ok(()) => Some(port),
                    Err(why) => return self.fail(why, cx),
                },
                Err(why) => return self.fail(why, cx),
            }
        };
        let key = self.key.clone();
        self.model
            .update(cx, |model, cx| model.forward(key, port, cx));
        window.close_dialog(cx);
    }

    fn fail(&mut self, why: String, cx: &mut Context<Self>) {
        self.error = Some(why);
        cx.notify();
    }
}

impl Render for ForwardDialog {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = Palette::of(cx);
        let this = cx.entity().downgrade();
        let forward = text_button(
            "forward-ok",
            "Forward",
            ButtonTone::Accent,
            true,
            &palette,
            move |_, window, cx| {
                this.update(cx, |this, cx| this.submit(window, cx)).ok();
            },
        );
        let cancel = text_button(
            "forward-cancel",
            "Cancel",
            ButtonTone::Accent,
            true,
            &palette,
            |_, window, cx| window.close_dialog(cx),
        );
        div()
            .flex()
            .flex_col()
            .gap(px(16.))
            .pb(px(4.))
            .child(field(
                "Local port",
                Input::new(&self.port).small().into_any_element(),
                &palette,
            ))
            .child(
                div()
                    .text_size(px(12.))
                    .text_color(palette.text2)
                    .child("Captain listens on 127.0.0.1 at this port while it runs. Leave it empty for a free port."),
            )
            .children(self.error.clone().map(|e| inline_error(e, &palette)))
            .child(
                div()
                    .flex()
                    .justify_end()
                    .gap(px(8.))
                    .child(cancel)
                    .child(forward),
            )
    }
}

/// Opens the dialog for the Service port `key`.
pub fn open(model: Entity<ForwardingModel>, key: ForwardKey, window: &mut Window, cx: &mut App) {
    let title = format!("Forward {}:{}", key.service, key.port);
    let form = cx.new(|cx| ForwardDialog::new(model, key, window, cx));
    window.open_dialog(cx, move |dialog, _, _| {
        dialog
            .title(title.clone())
            .w(px(420.))
            .overlay_closable(false)
            .child(form.clone())
    });
}
