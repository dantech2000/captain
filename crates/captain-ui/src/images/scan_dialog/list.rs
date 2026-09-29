use captain_core::model::{Severity, Vulnerability};
use gpui_kit::*;

use crate::theme::Palette;
use crate::widgets::pill;

/// The color of a severity.
pub fn severity_color(severity: Severity, palette: &Palette) -> Hsla {
    match severity {
        Severity::Critical => palette.red,
        Severity::High => palette.orange,
        Severity::Medium => palette.indigo,
        Severity::Low => palette.teal,
        Severity::Unknown => palette.gray,
    }
}

/// A virtual list of findings: severity, ID, package, installed and fixed versions,
/// and the title.
pub fn list(
    vulnerabilities: Vec<Vulnerability>,
    scroll: &UniformListScrollHandle,
    palette: &Palette,
) -> UniformList {
    let colors = *palette;
    uniform_list("scan-results", vulnerabilities.len(), move |range, _, _| {
        range.map(|ix| row(&vulnerabilities[ix], &colors)).collect()
    })
    .track_scroll(scroll)
    .flex_1()
}

fn row(vuln: &Vulnerability, palette: &Palette) -> Div {
    let color = severity_color(vuln.severity, palette);
    let fixed = match &vuln.fixed {
        Some(fixed) => format!("{} → {fixed}", vuln.installed),
        None => format!("{} · no fix", vuln.installed),
    };
    div()
        .h(px(46.))
        .px(px(8.))
        .flex()
        .items_center()
        .gap(px(10.))
        .border_b_1()
        .border_color(palette.sep)
        .text_size(px(12.))
        .child(div().w(px(64.)).flex().child(pill(
            vuln.severity.label(),
            color,
            palette.tint(color),
        )))
        .child(
            div()
                .w(px(150.))
                .flex_shrink_0()
                .flex()
                .flex_col()
                .child(
                    div()
                        .font_family(palette.mono())
                        .truncate()
                        .child(vuln.id.clone()),
                )
                .child(
                    div()
                        .text_color(palette.text2)
                        .truncate()
                        .child(vuln.package.clone()),
                ),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .child(
                    div()
                        .truncate()
                        .child(vuln.title.clone().unwrap_or_default()),
                )
                .child(
                    div()
                        .font_family(palette.mono())
                        .text_size(px(11.))
                        .text_color(palette.text3)
                        .truncate()
                        .child(fixed),
                ),
        )
}
