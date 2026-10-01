use gpui_kit::component::Sizable;
use gpui_kit::component::input::Input;
use gpui_kit::component::list::List;
use gpui_kit::*;

use super::step::{TemplateForm, TemplateStep};
use crate::help::HelpExt;
use crate::new_project::form::{
    compose_preview, field, footer, form_root, scroll_body, text_input, view_switch,
};
use crate::new_project::keys::Submit;
use crate::new_project::name_check::projects_dir;
use crate::theme::Palette;
use crate::widgets::inline_error;

impl Render for TemplateStep {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = Palette::of(cx);
        let weak = cx.entity().downgrade();
        let back = {
            let weak = weak.clone();
            move |window: &mut Window, cx: &mut App| {
                weak.update(cx, |this, cx| this.back(window, cx)).ok();
            }
        };
        let title = match &self.form {
            Some(form) => format!("New {} project", form.template.title),
            None => "Start from a template".into(),
        };
        let root = form_root("new-template")
            .on_action(cx.listener(|this, _: &Submit, window, cx| {
                if this.form.is_some() {
                    this.create(window, cx);
                }
            }))
            .child(heading(title, &palette));
        let Some(form) = &self.form else {
            return root
                .child(
                    div()
                        .h(px(300.))
                        .child(List::new(&self.list).search_placeholder("Search templates")),
                )
                .child(footer(
                    "Go back to the four ways to make a project.",
                    back,
                    ("Next", "Pick a template from the list first.".into(), false),
                    |_, _| {},
                    &palette,
                ));
        };
        let checked = self.values(cx);
        let body = if self.show_compose {
            let text = checked
                .as_ref()
                .map(|(template, values)| template.files(values)[0].text.clone())
                .map_err(Clone::clone);
            scroll_body("new-template-body").child(compose_preview(text, &palette))
        } else {
            scroll_body("new-template-body").child(fields(form, &palette))
        };
        let toggle = {
            let weak = weak.clone();
            move |compose: bool, _: &mut Window, cx: &mut App| {
                weak.update(cx, |this, cx| {
                    this.show_compose = compose;
                    cx.notify();
                })
                .ok();
            }
        };
        let create = {
            let weak = weak.clone();
            move |window: &mut Window, cx: &mut App| {
                weak.update(cx, |this, cx| this.create(window, cx)).ok();
            }
        };
        let dir = projects_dir(cx);
        let help = match &checked {
            Ok((_, values)) => format!(
                "Write {} and open it. Nothing starts until you apply it.",
                dir.join(&values.name).join("compose.yaml").display()
            ),
            Err(problem) => problem.clone(),
        };
        root.child(view_switch(self.show_compose, toggle, &palette))
            .child(body)
            .children(
                self.error
                    .clone()
                    .or_else(|| checked.as_ref().err().cloned())
                    .map(|error| inline_error(error, &palette)),
            )
            .child(footer(
                "Go back to the template list.",
                back,
                ("Create project", help, checked.is_ok()),
                create,
                &palette,
            ))
    }
}

fn heading(title: String, palette: &Palette) -> Div {
    div()
        .pr(px(32.))
        .text_size(px(18.))
        .font_weight(FontWeight::EXTRA_BOLD)
        .text_color(palette.text)
        .child(title)
}

fn fields(form: &TemplateForm, palette: &Palette) -> Div {
    let template = form.template;
    let mut fields = div().flex().flex_col().gap(px(14.)).child(field(
        "Project name",
        text_input(
            "template-name",
            &form.name,
            "The project's name and its folder. Lowercase letters, digits, - and _.",
        ),
        None,
        palette,
    ));
    for (index, (input, port)) in form.ports.iter().zip(template.ports).enumerate() {
        fields = fields.child(field(
            port.label,
            div().flex().items_center().gap(px(8.)).child(
                div().w(px(110.)).child(text_input(
                    ("template-port", index),
                    input,
                    format!(
                        "The port on this computer for port {} in the container. Captain suggests one that is free.",
                        port.container
                    ),
                )),
            )
            .child(
                div()
                    .text_size(px(12.))
                    .text_color(palette.text2)
                    .child(format!("→ {} in the container", port.container)),
            ),
            None,
            palette,
        ));
    }
    if let Some(user) = &form.user {
        fields = fields.child(field(
            "User name",
            text_input("template-user", user, "The first user. It goes in .env."),
            None,
            palette,
        ));
    }
    if let Some(password) = &form.password {
        fields = fields.child(field(
            "Password",
            div()
                .id("template-password")
                .child(Input::new(password).small().mask_toggle())
                .help("A random password. It goes in .env, which only you can read, never in compose.yaml."),
            None,
            palette,
        ));
    }
    fields
}
