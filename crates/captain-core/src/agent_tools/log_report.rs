//! The answer of the `logs` tool: the filtered lines of one container or a whole
//! project, newest kept, under the caps, secrets masked, between untrusted-output
//! delimiters.

use schemars::JsonSchema;
use serde::Serialize;

use super::log_query::{LogQuery, MAX_BYTES, MAX_LINE_CHARS, MAX_LINES};
use super::mask::mask_secrets;
use super::untrusted::{clean, wrap_untrusted};
use crate::model::LogLine;

/// A log line and the service or container it came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcedLine {
    pub source: String,
    pub line: LogLine,
}

/// The lines of a container or a project.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct LogReport {
    /// The container or project the lines come from.
    pub target: String,
    /// The lines in `output`.
    pub lines: usize,
    /// The lines that passed the filters, before the caps.
    pub matched: usize,
    /// True when the caps left out older lines or cut long ones.
    pub truncated: bool,
    /// What to ask next when `truncated` is true.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
    /// The Unix time of the newest line. Pass it as `since` to read newer lines.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub newest_time: Option<i64>,
    /// The lines, oldest first, as `HH:MM:SS [source |] text` in UTC, between
    /// untrusted-output delimiters. Secret-looking values are masked.
    pub output: String,
}

/// Filters `lines` with `query` and keeps the newest that fit the caps. With
/// `by_source`, the lines of several containers merge in time order and each shows
/// its source.
pub fn log_report(
    target: &str,
    mut lines: Vec<SourcedLine>,
    query: &LogQuery,
    by_source: bool,
) -> LogReport {
    lines.retain(|sourced| query.keeps(&sourced.line));
    if by_source {
        lines.sort_by_key(|sourced| sourced.line.precise_time());
    }
    let matched = lines.len();
    let mut kept = Vec::new();
    let mut bytes = 0;
    let mut cut = false;
    for sourced in lines.iter().rev().take(MAX_LINES) {
        let (text, was_cut) = render(sourced, by_source);
        if bytes + text.len() + 1 > MAX_BYTES {
            break;
        }
        bytes += text.len() + 1;
        cut |= was_cut;
        kept.push(text);
    }
    kept.reverse();
    let left_out = matched - kept.len();
    let hint = if left_out > 0 {
        Some(format!(
            "Showing the newest {} of {matched} lines. Narrow with grep or errors_only, \
             or pass a later since.",
            kept.len()
        ))
    } else {
        cut.then(|| format!("Lines longer than {MAX_LINE_CHARS} characters were cut."))
    };
    LogReport {
        target: target.to_string(),
        lines: kept.len(),
        matched,
        truncated: hint.is_some(),
        hint,
        newest_time: lines.iter().filter_map(|l| l.line.timestamp).max(),
        output: wrap_untrusted(target, &kept.join("\n")),
    }
}

/// One line of output, and whether it was cut.
fn render(sourced: &SourcedLine, by_source: bool) -> (String, bool) {
    let mut text = mask_secrets(&clean(&sourced.line.text));
    let cut = text.chars().count() > MAX_LINE_CHARS;
    if cut {
        text = text.chars().take(MAX_LINE_CHARS).collect();
        text.push('…');
    }
    let mut out = String::new();
    if let Some(clock) = sourced.line.clock(0) {
        out.push_str(&clock);
        out.push(' ');
    }
    if by_source {
        out.push_str(&sourced.source);
        out.push_str(" | ");
    }
    out.push_str(&text);
    (out, cut)
}

#[cfg(test)]
mod tests;
