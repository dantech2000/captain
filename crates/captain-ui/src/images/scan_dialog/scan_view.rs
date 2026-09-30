use std::sync::Arc;

use captain_core::Engine;
use captain_core::model::{ScanReport, Severity};
use gpui_kit::component::WindowExt;
use gpui_kit::*;

use super::ScanDialog;
use super::list::list;
use super::scan_state::ScanStatus;
use crate::theme::Palette;
use crate::widgets::{Segment, inline_error, section_note, segmented};

impl Render for ScanDialog {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = Palette::of(cx);
        let body = match &self.status {
            ScanStatus::Running(status) => div()
                .flex()
                .flex_col()
                .gap(px(4.))
                .child(section_note(
                    format!("Scanning {}...", self.reference),
                    &palette,
                ))
                .child(
                    div()
                        .text_size(px(12.))
                        .text_color(palette.text3)
                        .truncate()
                        .child(status.clone()),
                ),
            ScanStatus::Failed(error) => div().child(inline_error(error.clone(), &palette)),
            ScanStatus::Done(report) if report.vulnerabilities.is_empty() => {
                div().child(section_note(
                    format!("Trivy found no vulnerabilities in {}.", self.reference),
                    &palette,
                ))
            }
            ScanStatus::Done(report) => self.results(report, &palette, cx),
        };
        div().flex().flex_col().gap(px(12.)).pb(px(4.)).child(body)
    }
}

impl ScanDialog {
    fn results(&self, report: &ScanReport, palette: &Palette, cx: &mut Context<Self>) -> Div {
        let segment = |label: String, filter: Option<Severity>, cx: &mut Context<Self>| {
            let this = cx.entity().downgrade();
            Segment {
                label: label.into(),
                selected: self.filter == filter,
                help: match filter {
                    Some(severity) => format!(
                        "Show only the findings with {} severity.",
                        severity.label().to_lowercase()
                    ),
                    None => "Show all findings.".to_string(),
                }
                .into(),
                on_click: Box::new(move |_, cx| {
                    this.update(cx, |this, cx| this.set_filter(filter, cx)).ok();
                }),
            }
        };
        let mut segments = vec![segment(
            format!("All {}", report.vulnerabilities.len()),
            None,
            cx,
        )];
        for severity in Severity::ALL {
            let count = report.count(severity);
            if count > 0 || self.filter == Some(severity) {
                segments.push(segment(
                    format!("{} {count}", severity.label()),
                    Some(severity),
                    cx,
                ));
            }
        }
        let shown: Vec<_> = report.filtered(self.filter).into_iter().cloned().collect();
        div()
            .flex()
            .flex_col()
            .gap(px(10.))
            .child(
                div()
                    .flex()
                    .child(segmented("scan-filter", segments, palette)),
            )
            .child(
                div()
                    .h(px(420.))
                    .flex()
                    .flex_col()
                    .rounded(px(8.))
                    .border_1()
                    .border_color(palette.sep)
                    .child(list(shown, &self.scroll, palette)),
            )
    }
}

/// Opens the Scan dialog and starts scanning `reference`. Closing it stops the scan.
pub fn open(engine: Arc<dyn Engine>, reference: String, window: &mut Window, cx: &mut App) {
    let title = SharedString::from(format!("Scan {reference}"));
    let scan = cx.new(|cx| ScanDialog::new(engine, reference, cx));
    window.open_dialog(cx, move |dialog, _, _| {
        dialog.title(title.clone()).w(px(760.)).child(scan.clone())
    });
}
