//! The Tag dialog: adds a tag to an image, like `docker tag`.

use captain_core::model::ImageReference;
use gpui_kit::component::Sizable;
use gpui_kit::component::WindowExt;
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::*;

use super::ImagesState;
use super::field::field;
use crate::theme::Palette;
use crate::widgets::{ButtonTone, inline_error, text_button};

/// One field for the new reference. The dialog closes when the engine adds the tag.
pub struct TagDialog {
    state: Entity<ImagesState>,
    /// The image ID.
    image: String,
    input: Entity<InputState>,
    error: Option<String>,
    busy: bool,
    _subscription: Subscription,
    task: Option<Task<()>>,
}

impl TagDialog {
    fn new(
        state: Entity<ImagesState>,
        image: String,
        current: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("registry.example.com/team/app:1.0")
                .default_value(current)
        });
        let subscription = cx.subscribe_in(&input, window, |this, _, event, window, cx| {
            if let InputEvent::PressEnter { .. } = event {
                this.submit(window, cx);
            }
        });
        Self {
            state,
            image,
            input,
            error: None,
            busy: false,
            _subscription: subscription,
            task: None,
        }
    }

    fn submit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        let value = self.input.read(cx).value().to_string();
        let Some(target) = ImageReference::parse(&value).filter(|r| !r.tag.is_empty()) else {
            self.error = Some("Enter a reference with a tag, for example app:1.0.".into());
            cx.notify();
            return;
        };
        let Some(engine) = self.state.read(cx).engine.clone() else {
            return;
        };
        let target = target.to_string();
        self.busy = true;
        self.error = None;
        cx.notify();
        let tag = engine.tag_image(&self.image, &target);
        let state = self.state.clone();
        self.task = Some(cx.spawn_in(window, async move |this, cx| {
            let result = tag.await;
            this.update_in(cx, |this, window, cx| {
                this.busy = false;
                match result {
                    Ok(()) => {
                        state.update(cx, |state, cx| {
                            state.set_notice(format!("Tagged {target}"), cx);
                            state.reload_selected_detail(cx);
                        });
                        window.close_dialog(cx);
                    }
                    Err(error) => this.error = Some(format!("Tag failed: {error}")),
                }
                cx.notify();
            })
            .ok();
        }));
    }
}

impl Render for TagDialog {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = Palette::of(cx);
        let this = cx.entity().downgrade();
        let add = text_button(
            "add-tag",
            if self.busy { "Tagging..." } else { "Tag" },
            ButtonTone::Accent,
            !self.busy,
            &palette,
            move |_, window, cx| {
                this.update(cx, |this, cx| this.submit(window, cx)).ok();
            },
        );
        let cancel = text_button(
            "cancel-tag",
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
                "New reference",
                Input::new(&self.input).small().into_any_element(),
                &palette,
            ))
            .children(self.error.clone().map(|e| inline_error(e, &palette)))
            .child(
                div()
                    .flex()
                    .justify_end()
                    .gap(px(8.))
                    .child(cancel)
                    .child(add),
            )
    }
}

/// Opens the Tag dialog for the image `id`, prefilled with its first tag.
pub fn open(
    state: Entity<ImagesState>,
    id: String,
    current: String,
    window: &mut Window,
    cx: &mut App,
) {
    let form = cx.new(|cx| TagDialog::new(state, id, current, window, cx));
    window.open_dialog(cx, move |dialog, _, _| {
        dialog
            .title("Tag image")
            .w(px(460.))
            .overlay_closable(false)
            .child(form.clone())
    });
}
