use gpui_kit::component::Sizable;
use gpui_kit::component::list::List;
use gpui_kit::component::spinner::Spinner;
use gpui_kit::component::switch::Switch;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use super::form_state::{Detail, RunImage};
use crate::help::HelpExt;
use crate::new_project::form::{
    compose_preview, field, footer, form_root, scroll_body, text_input, view_switch,
};
use crate::new_project::keys::Submit;
use crate::new_project::name_check::projects_dir;
use crate::theme::Palette;
use crate::widgets::{ButtonTone, inline_error, text_button};

impl Render for RunImage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = Palette::of(cx);
        let weak = cx.entity().downgrade();
        let back = {
            let weak = weak.clone();
            move |window: &mut Window, cx: &mut App| {
                weak.update(cx, |this, cx| this.back(window, cx)).ok();
            }
        };
        let root = form_root("new-run-image")
            .on_action(cx.listener(|this, _: &Submit, window, cx| this.submit(window, cx)));
        if self.picking {
            return root
                .child(heading("Run an image".into()))
                .child(div().h(px(320.)).child(
                    List::new(&self.picker).search_placeholder("Search images, such as postgres"),
                ))
                .child(footer(
                    "Go back to the four ways to make a project.",
                    back,
                    ("Next", "Pick an image from the list first.".into(), false),
                    |_, _| {},
                    &palette,
                ));
        }
        let reference = self.reference(cx);
        let checked = self.check(cx);
        let body = if self.show_compose {
            let text = self.compose(cx).map(|(_, files)| files[0].text.clone());
            scroll_body("new-run-body").child(compose_preview(text, &palette))
        } else {
            scroll_body("new-run-body")
                .child(self.image_field(&palette, cx))
                .child(self.name_field(&palette, cx))
                .child(self.ports_field(&palette, cx))
                .child(self.env_field(&palette, cx))
                .child(self.volumes_field(&palette, cx))
                .child(field(
                    "Restart policy",
                    self.restart_control(&palette, cx),
                    None,
                    &palette,
                ))
                .when(!self.save_as_project, |body| {
                    body.child(self.auto_remove_control(cx))
                })
        };
        let label = if self.save_as_project {
            "Create project"
        } else {
            "Run"
        };
        let help = match (&checked, self.save_as_project) {
            (Err(problem), _) => problem.clone(),
            (Ok(()), true) => format!(
                "Write {} and open it. Nothing starts until you apply it.",
                projects_dir(cx)
                    .join(self.form(cx).name)
                    .join("compose.yaml")
                    .display()
            ),
            (Ok(()), false) => format!("Create and start a container from {reference}."),
        };
        let enabled = self.busy.is_none() && checked.is_ok();
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
        let submit = move |window: &mut Window, cx: &mut App| {
            weak.update(cx, |this, cx| this.submit(window, cx)).ok();
        };
        let problem = self.error.clone().or_else(|| checked.err());
        root.child(heading(format!("Run {reference}")))
            .child(self.project_switch(&palette, cx))
            .when(self.save_as_project, |root| {
                root.child(view_switch(self.show_compose, toggle, &palette))
            })
            .child(body)
            .children(self.busy.clone().map(|busy| {
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .text_size(px(12.))
                    .text_color(palette.text2)
                    .child(Spinner::new().xsmall().color(palette.text2))
                    .child(busy)
            }))
            .children(problem.map(|error| inline_error(error, &palette)))
            .child(footer(
                "Go back to the image list.",
                back,
                (label, help, enabled),
                submit,
                &palette,
            ))
    }
}

fn heading(title: String) -> Div {
    div()
        .pr(px(32.))
        .text_size(px(18.))
        .font_weight(FontWeight::EXTRA_BOLD)
        .truncate()
        .child(title)
}

impl RunImage {
    fn project_switch(&self, palette: &Palette, cx: &mut Context<Self>) -> Stateful<Div> {
        let this = cx.entity().downgrade();
        div()
            .id("run-save-project")
            .flex()
            .items_center()
            .gap(px(8.))
            .text_size(px(12.))
            .text_color(palette.text2)
            .child(
                Switch::new("run-save-project-switch")
                    .checked(self.save_as_project)
                    .label("Save as a project")
                    .on_click(move |checked, _, cx| {
                        let on = *checked;
                        this.update(cx, |this, cx| this.set_save_as_project(on, cx))
                            .ok();
                    }),
            )
            .help(
                "On: write a compose.yaml in a new project folder that you can edit and run again. \
                 Off: run a plain container now.",
            )
    }

    /// The tag field, the tags to pick from, and what Captain knows of the image.
    fn image_field(&self, palette: &Palette, cx: &mut Context<Self>) -> Div {
        let status = self.detail_status(palette, cx);
        let weak = cx.entity().downgrade();
        let chips = self.tags.iter().take(12).enumerate().map(|(ix, tag)| {
            let (this, tag) = (weak.clone(), tag.clone());
            div()
                .id(("run-tag", ix))
                .px(px(7.))
                .py(px(2.))
                .rounded(px(6.))
                .bg(palette.field)
                .text_size(px(11.))
                .cursor_pointer()
                .hover(|style| style.opacity(0.8))
                .child(tag.clone())
                .on_click(move |_, window, cx| {
                    let tag = tag.clone();
                    this.update(cx, |this, cx| {
                        this.tag
                            .update(cx, |input, cx| input.set_value(tag, window, cx));
                        this.tag_changed(window, cx);
                    })
                    .ok();
                })
                .help(format!("Use the tag {}.", self.tags[ix]))
        });
        let control = div()
            .flex()
            .flex_col()
            .gap(px(6.))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(
                        div()
                            .text_color(palette.text2)
                            .child(format!("{}:", self.repository)),
                    )
                    .child(div().w(px(160.)).child(text_input(
                        "run-tag-field",
                        &self.tag,
                        "The image's tag, such as 18 or latest. Empty means latest.",
                    )))
                    .children(self.pinned_digest(cx).map(|digest| {
                        let short: String = digest.chars().take(19).collect();
                        div()
                            .text_color(palette.text2)
                            .child(format!("@{short}\u{2026}"))
                    })),
            )
            .child(div().flex().flex_wrap().gap(px(4.)).children(chips))
            .children(status);
        field("Image tag", control, None, palette)
    }

    fn detail_status(&self, palette: &Palette, cx: &mut Context<Self>) -> Option<Div> {
        let line = div()
            .flex()
            .items_center()
            .gap(px(8.))
            .text_size(px(12.))
            .text_color(palette.text2);
        match &self.detail {
            Detail::Ready => None,
            Detail::Loading => Some(line.child("Reading the image's ports\u{2026}")),
            Detail::Failed(error) => Some(line.child(format!("Cannot read the image: {error}"))),
            Detail::Missing => {
                let this = cx.entity().downgrade();
                let pull = text_button(
                    "run-pull",
                    "Pull",
                    ButtonTone::Accent,
                    self.busy.is_none(),
                    palette,
                    move |_, window, cx| {
                        this.update(cx, |this, cx| this.pull(window, cx)).ok();
                    },
                )
                .help("Pull the image now and fill in the ports it exposes.");
                Some(
                    line.child(
                        "This engine does not have the image yet. Pull it to fill in its ports.",
                    )
                    .child(pull),
                )
            }
        }
    }

    fn name_field(&self, palette: &Palette, _: &mut Context<Self>) -> Div {
        let (label, help) = if self.save_as_project {
            (
                "Project name",
                "The project's name and its folder. Lowercase letters, digits, - and _.",
            )
        } else {
            (
                "Container name",
                "The container's name. Leave it empty for a random name.",
            )
        };
        field(
            label,
            text_input("run-name", &self.name, help),
            None,
            palette,
        )
    }
}
