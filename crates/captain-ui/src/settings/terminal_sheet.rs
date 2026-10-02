//! The terminal setup sheet: the checklist that makes the terminal's docker,
//! Compose, and Buildx use Captain Engine, and where each tool comes from now. See
//! docs/features/0037-settings-page.md.

use captain_core::cli_tools::{SHOWN_TOOLS, SetupSteps};
use gpui_kit::component::WindowExt;
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use super::{SettingsView, admin_access, path_step, store, terminal_steps};
use crate::help::HelpExt;
use crate::theme::Palette;
use crate::widgets::primary_button;

pub struct TerminalSheet {
    view: Entity<SettingsView>,
    /// True while "Where each tool comes from" is open.
    sources_open: bool,
    _observe: Subscription,
}

pub fn open(view: Entity<SettingsView>, window: &mut Window, cx: &mut App) {
    let sheet = cx.new(|cx| TerminalSheet {
        _observe: cx.observe(&view, |_, _, cx| cx.notify()),
        view,
        sources_open: false,
    });
    window.open_dialog(cx, move |dialog, _, _| {
        dialog.w(px(620.)).margin_top(px(110.)).child(sheet.clone())
    });
}

impl Render for TerminalSheet {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = Palette::of(cx);
        let settings = store::current(cx);
        let this = self.view.downgrade();
        let view = self.view.read(cx);
        let Some(facts) = view.terminal_facts(&settings, cx) else {
            return div().p(px(16.)).child("Checking your terminal\u{2026}");
        };
        let done = facts.steps.done();
        let sep = palette.sep;
        let steps: Vec<Div> = [
            Some(terminal_steps::links(
                view, &this, &settings, &facts, &palette,
            )),
            Some(path_step::render(view, &this, &facts, &palette)),
            Some(terminal_steps::context(view, &this, &facts, &palette)),
            admin_access::step(view, &this, &palette, cx),
        ]
        .into_iter()
        .flatten()
        .collect();
        let errors: Vec<SharedString> = [
            view.cli_tools.error.clone(),
            view.context_change.error.clone(),
        ]
        .into_iter()
        .flatten()
        .collect();
        let sources = self.sources_open.then(|| sources(view, &palette));
        div()
            .flex()
            .flex_col()
            .gap(px(16.))
            .child(header(done, &palette))
            .child(
                div()
                    .rounded(px(12.))
                    .border_1()
                    .border_color(sep)
                    .overflow_hidden()
                    .child(
                        div()
                            .id("terminal-sheet-steps")
                            .max_h(px(478.))
                            .overflow_y_scrollbar()
                            .flex()
                            .flex_col()
                            .children(steps.into_iter().enumerate().map(move |(ix, step)| {
                                step.when(ix > 0, |step| step.border_t_1().border_color(sep))
                            }))
                            .children(sources),
                    ),
            )
            .children(errors.into_iter().map(|error| {
                div()
                    .text_size(px(12.))
                    .text_color(palette.red)
                    .child(error)
            }))
            .child(self.footer(&palette, cx))
    }
}

fn header(done: usize, palette: &Palette) -> Div {
    let all = done == SetupSteps::COUNT;
    div()
        .flex()
        .items_start()
        .gap(px(12.))
        // Leave room for the dialog's close button in the corner.
        .pr(px(32.))
        .child(
            div()
                .flex_1()
                .flex()
                .flex_col()
                .gap(px(4.))
                .child(
                    div()
                        .text_size(px(18.))
                        .font_weight(FontWeight::EXTRA_BOLD)
                        .child("Use Captain from your terminal"),
                )
                .child(
                    div()
                        .text_color(palette.text2)
                        .child("docker, Compose, Buildx, the keychain helper, kubectl, and helm."),
                ),
        )
        .child(
            div()
                .pt(px(4.))
                .text_size(px(12.))
                .font_weight(FontWeight::BOLD)
                .text_color(if all {
                    palette.green
                } else {
                    palette.warn_text
                })
                .child(format!("{done} of {} done", SetupSteps::COUNT)),
        )
}

/// Where a new terminal finds each tool now.
fn sources(view: &SettingsView, palette: &Palette) -> Div {
    let rows = SHOWN_TOOLS.iter().map(|tool| {
        let source = match &view.cli_tools.resolved {
            Some(Ok(tools)) => tools
                .iter()
                .find(|(name, _)| name == tool)
                .map_or("Not found".into(), |(_, source)| source.label()),
            Some(Err(error)) => error.clone(),
            None => "Checking\u{2026}".into(),
        };
        div()
            .flex()
            .gap(px(12.))
            .text_size(px(12.))
            .child(
                div()
                    .w(px(220.))
                    .font_family(palette.mono())
                    .child(tool.to_string()),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .text_color(palette.text2)
                    .child(source),
            )
    });
    div()
        .flex()
        .flex_col()
        .gap(px(6.))
        .px(px(16.))
        .py(px(12.))
        .border_t_1()
        .border_color(palette.sep)
        .bg(palette.field)
        .children(rows)
}

impl TerminalSheet {
    fn footer(&self, palette: &Palette, cx: &mut Context<Self>) -> Div {
        let view = self.view.clone();
        let open = self.sources_open;
        div()
            .flex()
            .items_center()
            .gap(px(10.))
            .child(
                div()
                    .id("terminal-sheet-sources")
                    .text_size(px(12.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(palette.link)
                    .cursor_pointer()
                    .on_click(cx.listener(|sheet, _, _, cx| {
                        sheet.sources_open = !sheet.sources_open;
                        cx.notify();
                    }))
                    .child(if open {
                        "Hide where each tool comes from"
                    } else {
                        "Where each tool comes from now"
                    })
                    .help("Show where a new terminal finds docker, Compose, the keychain helper, captain, kubectl, and helm."),
            )
            .child(div().flex_1())
            .child(
                div()
                    .id("terminal-sheet-check")
                    .h(px(32.))
                    .px(px(12.))
                    .flex()
                    .items_center()
                    .rounded(px(8.))
                    .border_1()
                    .border_color(palette.border_strong)
                    .text_size(px(12.5))
                    .font_weight(FontWeight::SEMIBOLD)
                    .cursor_pointer()
                    .hover(|style| style.opacity(0.85))
                    .on_click(move |_, _, cx| {
                        view.update(cx, |view, cx| {
                            view.refresh_tools(cx);
                            view.rescan(cx);
                        });
                    })
                    .child("Check again")
                    .help("Look again at the links, the shell files, the tools, and the docker context."),
            )
            .child(primary_button(
                "terminal-sheet-done",
                "Done",
                "Close the terminal setup.",
                true,
                palette,
                |_, window, cx| window.close_dialog(cx),
            ))
    }
}
