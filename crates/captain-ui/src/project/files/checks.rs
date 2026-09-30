//! Checks of the unsaved text as the user types: `docker compose config` for a
//! Compose file, BuildKit's build checks for a Dockerfile. The results show on
//! their lines in the editor and in the list under it.

use std::time::Duration;

use captain_core::project_files::{FileKind, LineProblem, Severity};
use captain_core::{EngineError, EngineFuture};
use gpui_kit::component::highlighter::{Diagnostic, DiagnosticSeverity};
use gpui_kit::component::input::Position;
use gpui_kit::*;

use super::FileEditor;

/// How long typing must pause before a check runs.
const CHECK_DELAY: Duration = Duration::from_millis(700);

impl FileEditor {
    /// Checks the text after a pause. A new edit drops the waiting or running
    /// check, which kills its command.
    pub(super) fn schedule_check(&mut self, cx: &mut Context<Self>) {
        if self.runner.is_none() {
            return;
        }
        self.checking = true;
        self.check = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(CHECK_DELAY).await;
            let Ok(Some(check)) = this.update(cx, |this, cx| this.start_check(cx)) else {
                return;
            };
            let result = check.await;
            this.update(cx, |this, cx| this.show_problems(result, cx))
                .ok();
        }));
    }

    fn start_check(&self, cx: &App) -> Option<EngineFuture<Vec<LineProblem>>> {
        let runner = self.runner.as_ref()?;
        let text = self.editor.read(cx).value().to_string();
        Some(match self.file.kind {
            FileKind::Compose => runner.check_compose(&self.project, &self.file.path, text),
            FileKind::Dockerfile => {
                let context = self.file.context.as_deref()?;
                runner.check_dockerfile(context, text)
            }
        })
    }

    fn show_problems(
        &mut self,
        result: Result<Vec<LineProblem>, EngineError>,
        cx: &mut Context<Self>,
    ) {
        self.checking = false;
        let (problems, error) = match result {
            Ok(problems) => (problems, None),
            Err(error) => (Vec::new(), Some(error.to_string())),
        };
        self.problems = problems;
        self.check_error = error;
        let problems = self.problems.clone();
        let origin = match self.file.kind {
            FileKind::Compose => "docker compose",
            FileKind::Dockerfile => "build check",
        };
        self.editor.update(cx, |state, cx| {
            let text = state.value();
            let Some(set) = state.diagnostics_mut() else {
                return;
            };
            set.clear();
            for problem in &problems {
                let Some(line) = problem.line else {
                    continue;
                };
                let width = text.lines().nth(line).map_or(0, |l| l.chars().count());
                let start = Position::new(line as u32, 0);
                let end = Position::new(line as u32, width as u32);
                let severity = match problem.severity {
                    Severity::Error => DiagnosticSeverity::Error,
                    Severity::Warning => DiagnosticSeverity::Warning,
                };
                set.push(
                    Diagnostic::new(start..end, problem.message.clone())
                        .with_severity(severity)
                        .with_source(origin),
                );
            }
            cx.notify();
        });
        cx.notify();
    }
}
