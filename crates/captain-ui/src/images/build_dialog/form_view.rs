use gpui_kit::assets::IconName;
use gpui_kit::component::Sizable;
use gpui_kit::component::WindowExt;
use gpui_kit::component::input::{Input, InputState};
use gpui_kit::*;

use super::BuildDialog;
use super::form_state::BuildStatus;
use super::log_view::log_view;
use crate::images::field::field;
use crate::theme::Palette;
use crate::widgets::{ButtonTone, icon_button, inline_error, text_button};

impl Render for BuildDialog {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = Palette::of(cx);
        let this = cx.entity().downgrade();
        let choose_context = text_button(
            "choose-context",
            "Choose...",
            ButtonTone::Accent,
            true,
            &palette,
            move |_, window, cx| {
                this.update(cx, |this, cx| this.choose_context(window, cx))
                    .ok();
            },
        );
        let this = cx.entity().downgrade();
        let choose_dockerfile = text_button(
            "choose-dockerfile",
            "Choose...",
            ButtonTone::Accent,
            true,
            &palette,
            move |_, window, cx| {
                this.update(cx, |this, cx| this.choose_dockerfile(window, cx))
                    .ok();
            },
        );
        let show_log = !self.log.is_empty() || self.status != BuildStatus::Idle;
        div()
            .flex()
            .flex_col()
            .gap(px(14.))
            .pb(px(4.))
            .child(field(
                "Context folder",
                with_button(&self.context, choose_context).into_any_element(),
                &palette,
            ))
            .child(field(
                "Dockerfile",
                with_button(&self.dockerfile, choose_dockerfile).into_any_element(),
                &palette,
            ))
            .child(
                div()
                    .flex()
                    .gap(px(12.))
                    .child(div().flex_1().child(field(
                        "Tag",
                        Input::new(&self.tag).small().into_any_element(),
                        &palette,
                    )))
                    .child(div().flex_1().child(field(
                        "Target",
                        Input::new(&self.target).small().into_any_element(),
                        &palette,
                    ))),
            )
            .child(self.args_field(&palette, cx))
            .children(self.error.clone().map(|e| inline_error(e, &palette)))
            .children(show_log.then(|| log_view(&self.log, &self.scroll, &palette)))
            .children(status_line(&self.status, &palette))
            .child(self.footer(&palette, cx))
    }
}

impl BuildDialog {
    fn args_field(&self, palette: &Palette, cx: &mut Context<Self>) -> Div {
        let rows = self.args.iter().enumerate().map(|(ix, input)| {
            let this = cx.entity().downgrade();
            div()
                .flex()
                .items_center()
                .gap(px(6.))
                .child(div().flex_1().min_w_0().child(Input::new(input).small()))
                .child(icon_button(
                    ("remove-build-arg", ix),
                    IconName::Minus,
                    palette,
                    move |_, _, cx| {
                        this.update(cx, |this, cx| this.remove_arg(ix, cx)).ok();
                    },
                ))
        });
        let this = cx.entity().downgrade();
        let add = text_button(
            "add-build-arg",
            "Add argument",
            ButtonTone::Accent,
            true,
            palette,
            move |_, window, cx| {
                this.update(cx, |this, cx| this.add_arg(window, cx)).ok();
            },
        );
        let body = div()
            .flex()
            .flex_col()
            .gap(px(6.))
            .children(rows)
            .child(div().flex().child(add));
        field("Build arguments", body.into_any_element(), palette)
    }

    fn footer(&self, palette: &Palette, cx: &mut Context<Self>) -> Div {
        let this = cx.entity().downgrade();
        let running = self.is_running();
        let build = text_button(
            "start-build",
            if running { "Building..." } else { "Build" },
            ButtonTone::Accent,
            !running,
            palette,
            move |_, _, cx| {
                this.update(cx, |this, cx| this.submit(cx)).ok();
            },
        );
        let close = text_button(
            "close-build",
            if running { "Stop" } else { "Close" },
            ButtonTone::Accent,
            true,
            palette,
            |_, window, cx| window.close_dialog(cx),
        );
        div()
            .flex()
            .justify_end()
            .gap(px(8.))
            .child(close)
            .child(build)
    }
}

/// An input with a button on its right.
fn with_button(input: &Entity<InputState>, button: Stateful<Div>) -> Div {
    div()
        .flex()
        .items_center()
        .gap(px(8.))
        .child(div().flex_1().min_w_0().child(Input::new(input).small()))
        .child(button)
}

/// "Building...", "Built app:dev" in green, or the error in red.
fn status_line(status: &BuildStatus, palette: &Palette) -> Option<Div> {
    let (text, color) = match status {
        BuildStatus::Idle => return None,
        BuildStatus::Running => ("Building...".to_string(), palette.text2),
        BuildStatus::Built(tag) => (format!("Built {tag}"), palette.green),
        BuildStatus::Failed(error) => (format!("Build failed: {error}"), palette.red),
    };
    Some(div().text_size(px(12.)).text_color(color).child(text))
}
