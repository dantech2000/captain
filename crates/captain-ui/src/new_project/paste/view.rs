use gpui_kit::component::input::Textarea;
use gpui_kit::*;

use super::step::PasteStep;
use crate::help::HelpExt;
use crate::new_project::form::{
    compose_preview, field, footer, form_root, scroll_body, text_input, view_switch,
};
use crate::new_project::keys::Submit;
use crate::new_project::name_check::projects_dir;
use crate::theme::Palette;
use crate::widgets::inline_error;

impl Render for PasteStep {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = Palette::of(cx);
        let weak = cx.entity().downgrade();
        let checked = self.compose(cx);
        let body = if self.show_compose {
            let text = checked
                .as_ref()
                .map(|(_, text)| text.clone())
                .map_err(Clone::clone);
            scroll_body("new-paste-body").child(compose_preview(text, &palette))
        } else {
            scroll_body("new-paste-body")
                .child(field(
                    "docker run command",
                    div()
                        .id("paste-command")
                        .child(Textarea::new(&self.command))
                        .help("Paste a docker run command. Lines that end in \\ are joined."),
                    None,
                    &palette,
                ))
                .children(self.result(&palette))
                .child(field(
                    "Project name",
                    text_input(
                        "paste-name",
                        &self.name,
                        "The project's name and its folder. Lowercase letters, digits, - and _.",
                    ),
                    None,
                    &palette,
                ))
        };
        let (back, toggle, create) = {
            let (b, t, c) = (weak.clone(), weak.clone(), weak);
            (
                move |window: &mut Window, cx: &mut App| {
                    b.update(cx, |this, cx| this.back(window, cx)).ok();
                },
                move |compose: bool, _: &mut Window, cx: &mut App| {
                    t.update(cx, |this, cx| {
                        this.show_compose = compose;
                        cx.notify();
                    })
                    .ok();
                },
                move |window: &mut Window, cx: &mut App| {
                    c.update(cx, |this, cx| this.create(window, cx)).ok();
                },
            )
        };
        let help = match &checked {
            Ok((name, _)) => format!(
                "Write {} and open it. Nothing starts until you apply it.",
                projects_dir(cx).join(name).join("compose.yaml").display()
            ),
            Err(problem) => problem.clone(),
        };
        let problem = self.error.clone().or_else(|| {
            checked
                .as_ref()
                .err()
                .cloned()
                .filter(|_| self.conversion.is_some())
        });
        form_root("new-paste")
            .on_action(cx.listener(|this, _: &Submit, window, cx| this.create(window, cx)))
            .child(
                div()
                    .pr(px(32.))
                    .text_size(px(18.))
                    .font_weight(FontWeight::EXTRA_BOLD)
                    .child("Paste a docker run command"),
            )
            .child(view_switch(self.show_compose, toggle, &palette))
            .child(body)
            .children(problem.map(|error| inline_error(error, &palette)))
            .child(footer(
                "Go back to the four ways to make a project.",
                back,
                ("Create project", help, checked.is_ok()),
                create,
                &palette,
            ))
    }
}

impl PasteStep {
    /// What the conversion made of the command: the service, and a line for each
    /// warning.
    fn result(&self, palette: &Palette) -> Option<Div> {
        let Some(Ok(conversion)) = &self.conversion else {
            return None;
        };
        let summary = format!(
            "One service, {}, from {}.",
            conversion.service_name, conversion.service.image
        );
        Some(
            div()
                .flex()
                .flex_col()
                .gap(px(4.))
                .text_size(px(12.))
                .child(div().text_color(palette.text2).child(summary))
                .children(conversion.warnings.iter().map(|warning| {
                    div()
                        .text_color(palette.warn_text)
                        .child(format!("\u{26a0} {warning}"))
                })),
        )
    }
}
