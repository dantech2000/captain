use captain_core::diagnostics::CheckState;
use gpui_kit::*;

use super::{DiagnosticsModel, check_row, diagnostics_model, engine_card, troubleshooting};
use crate::engine_host::{host_model, summary as host_summary};
use crate::help::HelpExt;
use crate::settings;
use crate::theme::Palette;
use crate::widgets::{ButtonTone, inline_notice, page_header, settings_card, text_button};
use crate::workspace::Workspace;

/// The Diagnostics page: the engine with its controls, the checks with their fixes,
/// "Run again", and the troubleshooting actions.
pub struct DiagnosticsView {
    workspace: Entity<Workspace>,
    model: Option<Entity<DiagnosticsModel>>,
    _subscriptions: Vec<Subscription>,
}

impl DiagnosticsView {
    pub fn new(workspace: Entity<Workspace>, cx: &mut Context<Self>) -> Self {
        let model = diagnostics_model(cx);
        let mut subscriptions = vec![
            cx.observe_global::<settings::SettingsStore>(|_, cx| cx.notify()),
            cx.observe(&workspace, |_, _, cx| cx.notify()),
        ];
        subscriptions.extend(
            model
                .as_ref()
                .map(|model| cx.observe(model, |_, _, cx| cx.notify())),
        );
        subscriptions.extend(host_model(cx).map(|host| cx.observe(&host, |_, _, cx| cx.notify())));
        Self {
            workspace,
            model,
            _subscriptions: subscriptions,
        }
    }
}

/// "2 failed · 1 warning · checked at 10:42", or what is going on.
fn summary(model: &DiagnosticsModel) -> String {
    let count = |state| {
        model
            .checks()
            .iter()
            .filter(|check| check.state == state)
            .count()
    };
    let (failed, warnings) = (count(CheckState::Failed), count(CheckState::Warning));
    let mut parts = Vec::new();
    match (failed, warnings) {
        (0, 0) => parts.push("No problems found".to_string()),
        _ => {
            if failed > 0 {
                parts.push(format!("{failed} failed"));
            }
            if warnings > 0 {
                let noun = if warnings == 1 { "warning" } else { "warnings" };
                parts.push(format!("{warnings} {noun}"));
            }
        }
    }
    match model.checked_at() {
        _ if model.is_running() => parts = vec!["Running the checks...".into()],
        Some(time) => parts.push(format!("checked at {time}")),
        None => {}
    }
    parts.join(" · ")
}

impl Render for DiagnosticsView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = Palette::of(cx);
        let Some(handle) = self.model.clone() else {
            return div()
                .size_full()
                .p(px(24.))
                .child(inline_notice("Diagnostics are not available.", &palette));
        };
        let engine = engine_card::render(
            self.workspace.read(cx),
            host_summary(cx).as_ref(),
            host_model(cx).filter(|model| model.read(cx).supported()),
            &palette,
        );
        let model = handle.read(cx);
        let debug_logging = settings::current(cx).debug_logging;
        let engine_dir = model.setup.engine_dir.clone();
        let rows: Vec<AnyElement> = model
            .checks()
            .iter()
            .map(|check| check_row::render(check, engine_dir.clone(), &palette))
            .collect();
        let run_again = text_button(
            "diagnostics-run",
            "Run again",
            ButtonTone::Accent,
            !model.is_running(),
            &palette,
            move |_, _, cx| handle.update(cx, |model, cx| model.run(true, cx)),
        )
        .help("Run every check again: the engine, its socket, and the tools.")
        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation());

        div()
            .size_full()
            .flex()
            .flex_col()
            .child(page_header(
                "diagnostics-header",
                "Diagnostics",
                summary(model),
                Some(run_again.into_any_element()),
                &palette,
            ))
            .child(
                div()
                    .id("diagnostics-scroll")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .px(px(24.))
                    .pt(px(4.))
                    .pb(px(24.))
                    .child(
                        div()
                            .w_full()
                            .max_w(px(720.))
                            .flex()
                            .flex_col()
                            .gap(px(22.))
                            .child(engine)
                            .child(settings_card("Checks", rows, &palette))
                            .child(troubleshooting::render(model, debug_logging, &palette)),
                    ),
            )
    }
}
