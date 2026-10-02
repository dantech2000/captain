use std::sync::Arc;
use std::time::Duration;

use captain_core::Engine;
use captain_core::model::ProcessTable;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use super::section;
use crate::theme::Palette;
use crate::widgets::skeleton_lines;

/// How often the table asks the engine again.
const REFRESH: Duration = Duration::from_secs(2);
/// The columns the table shows when the engine returns them. The rest do not fit
/// the inspector's width.
const COLUMNS: [&str; 4] = ["PID", "UID", "TIME", "CMD"];

/// The Processes table under the Stats charts, like `docker top`. It polls only
/// while the Stats tab shows and the container runs.
pub struct ProcessList {
    engine: Option<Arc<dyn Engine>>,
    /// The container's ID, and whether it runs.
    target: Option<(String, bool)>,
    active: bool,
    table: Option<Result<ProcessTable, String>>,
    task: Option<Task<()>>,
}

impl ProcessList {
    pub fn new() -> Self {
        Self {
            engine: None,
            target: None,
            active: false,
            table: None,
            task: None,
        }
    }

    /// Follows the inspector's container.
    pub fn set_target(
        &mut self,
        target: Option<(String, bool)>,
        engine: Option<Arc<dyn Engine>>,
        cx: &mut Context<Self>,
    ) {
        if target != self.target {
            self.table = None;
        }
        self.target = target;
        self.engine = engine;
        self.restart(cx);
    }

    /// Starts polling while the Stats tab shows, and stops when it hides.
    pub fn set_active(&mut self, active: bool, cx: &mut Context<Self>) {
        if active != self.active {
            self.active = active;
            self.restart(cx);
        }
    }

    fn restart(&mut self, cx: &mut Context<Self>) {
        self.task = None;
        let (Some(engine), Some((id, true)), true) =
            (self.engine.clone(), self.target.clone(), self.active)
        else {
            return;
        };
        self.task = Some(cx.spawn(async move |this, cx| {
            loop {
                let result = engine.top(&id).await.map_err(|e| e.to_string());
                if this
                    .update(cx, |this, cx| {
                        this.table = Some(result);
                        cx.notify();
                    })
                    .is_err()
                {
                    return;
                }
                cx.background_executor().timer(REFRESH).await;
            }
        }));
    }
}

impl Render for ProcessList {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = Palette::of(cx);
        let running = self.target.as_ref().is_some_and(|t| t.1);
        let body = match (&self.table, running) {
            (_, false) => message("Processes appear while the container runs.", &palette),
            (None, true) => skeleton_lines(3),
            (Some(Err(error)), true) => message(error.clone(), &palette).text_color(palette.red),
            (Some(Ok(table)), true) => table_view(table, &palette),
        };
        div()
            .flex()
            .flex_col()
            .gap(px(8.))
            .px(px(20.))
            .pt(px(16.))
            .child(section::heading("Processes", &palette))
            .child(body)
    }
}

fn message(text: impl Into<SharedString>, palette: &Palette) -> Div {
    div()
        .text_size(px(12.))
        .text_color(palette.text3)
        .child(text.into())
}

fn table_view(table: &ProcessTable, palette: &Palette) -> Div {
    let mut columns: Vec<usize> = COLUMNS
        .iter()
        .filter_map(|name| table.titles.iter().position(|t| t == name))
        .collect();
    if columns.is_empty() {
        columns = (0..table.titles.len()).collect();
    }
    let last = table.titles.len().saturating_sub(1);
    let cell = move |ix: usize, text: String| {
        let wide = ix == last || table.titles.get(ix).is_some_and(|t| t == "CMD");
        div()
            .when(wide, |this| this.flex_1().min_w_0().truncate())
            .when(!wide, |this| this.w(px(64.)).flex_shrink_0().truncate())
            .child(text)
    };
    let line = |values: &[String]| {
        div().flex().gap(px(8.)).px(px(10.)).py(px(5.)).children(
            columns
                .iter()
                .map(|&ix| cell(ix, values.get(ix).cloned().unwrap_or_default())),
        )
    };
    div()
        .flex()
        .flex_col()
        .rounded(px(9.))
        .border_1()
        .border_color(palette.sep)
        .overflow_hidden()
        .font_family(palette.mono())
        .text_size(px(11.))
        .child(
            line(&table.titles)
                .text_color(palette.text3)
                .border_b_1()
                .border_color(palette.sep),
        )
        .children(
            table.rows.iter().enumerate().map(|(ix, row)| {
                line(row).when(ix > 0, |r| r.border_t_1().border_color(palette.sep))
            }),
        )
}
