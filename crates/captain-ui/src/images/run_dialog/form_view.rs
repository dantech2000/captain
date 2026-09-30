use captain_core::model::RestartPolicy;
use gpui_kit::assets::IconName;
use gpui_kit::component::Sizable;
use gpui_kit::component::WindowExt;
use gpui_kit::component::checkbox::Checkbox;
use gpui_kit::component::input::Input;
use gpui_kit::*;

use super::RunDialog;
use crate::images::field::field;
use crate::theme::Palette;
use crate::widgets::{
    ButtonTone, Segment, icon_button, inline_error, section_note, segmented, text_button,
};

impl Render for RunDialog {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = Palette::of(cx);
        div()
            .flex()
            .flex_col()
            .gap(px(16.))
            .pb(px(4.))
            .child(field(
                "Name",
                Input::new(&self.name).small().into_any_element(),
                &palette,
            ))
            .child(self.ports_field(&palette))
            .child(self.env_field(&palette, cx))
            .child(field(
                "Restart policy",
                self.restart_control(&palette, cx).into_any_element(),
                &palette,
            ))
            .child(self.auto_remove_control(cx))
            .children(
                self.error
                    .clone()
                    .map(|error| inline_error(error, &palette)),
            )
            .child(self.footer(&palette, cx))
    }
}

impl RunDialog {
    fn ports_field(&self, palette: &Palette) -> Div {
        let body = if self.ports.is_empty() {
            section_note("The image exposes no ports.", palette)
        } else {
            div()
                .flex()
                .flex_col()
                .gap(px(6.))
                .children(self.ports.iter().map(|(container, input)| {
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .child(div().w(px(110.)).child(Input::new(input).small()))
                        .child(
                            div()
                                .text_size(px(12.))
                                .text_color(palette.text2)
                                .child(format!("→ {container} in the container")),
                        )
                }))
        };
        field("Host ports", body.into_any_element(), palette)
    }

    fn env_field(&self, palette: &Palette, cx: &mut Context<Self>) -> Div {
        let rows = self.env.iter().enumerate().map(|(ix, input)| {
            let this = cx.entity().downgrade();
            div()
                .flex()
                .items_center()
                .gap(px(6.))
                .child(div().flex_1().min_w_0().child(Input::new(input).small()))
                .child(icon_button(
                    ("remove-env", ix),
                    IconName::Minus,
                    "Remove this environment variable.",
                    palette,
                    move |_, _, cx| {
                        this.update(cx, |this, cx| this.remove_env(ix, cx)).ok();
                    },
                ))
        });
        let add = {
            let this = cx.entity().downgrade();
            text_button(
                "add-env",
                "Add variable",
                ButtonTone::Accent,
                true,
                palette,
                move |_, window, cx| {
                    this.update(cx, |this, cx| this.add_env(window, cx)).ok();
                },
            )
        };
        let body = div()
            .flex()
            .flex_col()
            .gap(px(6.))
            .children(rows)
            .child(div().flex().child(add));
        field("Environment", body.into_any_element(), palette)
    }

    fn restart_control(&self, palette: &Palette, cx: &mut Context<Self>) -> Div {
        let segments = RestartPolicy::ALL
            .into_iter()
            .map(|policy| {
                let this = cx.entity().downgrade();
                Segment {
                    label: policy.name().into(),
                    selected: self.restart == policy,
                    on_click: Box::new(move |_, cx| {
                        this.update(cx, |this, cx| this.set_restart(policy, cx))
                            .ok();
                    }),
                }
            })
            .collect();
        div()
            .flex()
            .child(segmented("restart-policy", segments, palette))
    }

    fn auto_remove_control(&self, cx: &mut Context<Self>) -> Checkbox {
        let this = cx.entity().downgrade();
        Checkbox::new("auto-remove")
            .label("Remove the container when it exits")
            .checked(self.auto_remove)
            .on_click(move |checked, _, cx| {
                this.update(cx, |this, cx| this.set_auto_remove(*checked, cx))
                    .ok();
            })
    }

    fn footer(&self, palette: &Palette, cx: &mut Context<Self>) -> Div {
        let this = cx.entity().downgrade();
        let run = text_button(
            "run-container",
            if self.busy { "Starting..." } else { "Run" },
            ButtonTone::Accent,
            !self.busy,
            palette,
            move |_, window, cx| {
                this.update(cx, |this, cx| this.submit(window, cx)).ok();
            },
        );
        let cancel = text_button(
            "cancel-run",
            "Cancel",
            ButtonTone::Accent,
            true,
            palette,
            |_, window, cx| window.close_dialog(cx),
        );
        div()
            .flex()
            .justify_end()
            .gap(px(8.))
            .child(cancel)
            .child(run)
    }
}
