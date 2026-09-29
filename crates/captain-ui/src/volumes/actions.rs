use std::time::Duration;

use captain_core::store::validate_name;
use gpui_kit::component::input::{InputEvent, InputState};
use gpui_kit::*;

use super::VolumesView;

impl VolumesView {
    /// The name field. It needs a window, so the first render creates it.
    pub(super) fn input(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<InputState> {
        if let Some(input) = &self.input {
            return input.clone();
        }
        let input = cx.new(|cx| InputState::new(window, cx).placeholder("New volume name"));
        let events = cx.subscribe_in(&input, window, |this, input, event, window, cx| {
            match event {
                InputEvent::Change => {
                    // Hint while typing, but not for an empty field.
                    let name = input.read(cx).value();
                    this.name_hint = validate_name(name.trim())
                        .err()
                        .filter(|_| !name.is_empty())
                        .map(|error| error.to_string().into());
                    cx.notify();
                }
                InputEvent::PressEnter { .. } => this.create(window, cx),
                _ => {}
            }
        });
        self.subscriptions.push(events);
        self.input = Some(input.clone());
        input
    }

    /// Creates a volume with the name in the field, then selects it.
    pub(super) fn create(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let (Some(engine), Some(input)) = (self.engine.clone(), self.input.clone()) else {
            return;
        };
        if self.creating {
            return;
        }
        let name = input.read(cx).value().trim().to_string();
        if let Err(error) = validate_name(&name) {
            self.name_hint = Some(error.to_string().into());
            cx.notify();
            return;
        }
        self.creating = true;
        self.error = None;
        self.notice = None;
        cx.notify();
        cx.spawn_in(window, async move |this, cx| {
            let result = engine.create_volume(&name).await;
            this.update_in(cx, |this, window, cx| {
                this.creating = false;
                match result {
                    Ok(()) => {
                        input.update(cx, |input, cx| input.set_value("", window, cx));
                        this.name_hint = None;
                        this.selected = Some(name);
                        this.reload(Duration::ZERO, cx);
                    }
                    Err(error) => this.error = Some(format!("Create failed: {error}")),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    /// Removes a volume. The engine refuses one that a container uses.
    pub(super) fn remove(&mut self, name: String, cx: &mut Context<Self>) {
        let Some(engine) = self.engine.clone() else {
            return;
        };
        if !self.removing.insert(name.clone()) {
            return;
        }
        self.error = None;
        self.notice = None;
        cx.notify();
        cx.spawn(async move |this, cx| {
            let result = engine.remove_volume(&name).await;
            this.update(cx, |this, cx| {
                this.removing.remove(&name);
                match result {
                    Ok(()) => this.reload(Duration::ZERO, cx),
                    Err(error) => this.error = Some(format!("Remove failed: {error}")),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    /// Selects a volume and loads the containers that use it. A click on the selected
    /// volume closes the panel.
    pub(super) fn select(&mut self, name: String, cx: &mut Context<Self>) {
        if self.selected.as_deref() == Some(name.as_str()) {
            self.selected = None;
        } else {
            self.selected = Some(name);
        }
        self.load_users(cx);
        cx.notify();
    }
}
