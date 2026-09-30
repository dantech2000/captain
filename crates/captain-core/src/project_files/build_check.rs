use serde_json::Value;

use super::LineProblem;

/// Reads the output of `docker build --call check,format=json -q -f - <context>`:
/// each warning on its line, and a `buildError` as an error. A base image that
/// cannot be looked up is only a warning: the Dockerfile itself may be fine. Lines
/// in the output are 1-based. Fails when the output is not the check's JSON, for
/// example from a Buildx older than 0.15.
pub fn build_check_problems(stdout: &str) -> Result<Vec<LineProblem>, String> {
    let start = stdout
        .find('{')
        .ok_or_else(|| "the build check printed no result".to_string())?;
    let report: Value = serde_json::from_str(&stdout[start..])
        .map_err(|error| format!("cannot read the build check result: {error}"))?;
    let mut problems: Vec<LineProblem> = report
        .get("warnings")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|warning| {
            let text = |key| warning.get(key).and_then(Value::as_str).unwrap_or("");
            let detail = match text("detail") {
                "" => text("description"),
                detail => detail,
            };
            let message = match text("ruleName") {
                "" => detail.to_string(),
                rule => format!("{detail} ({rule})"),
            };
            LineProblem::warning(first_line(warning), message)
        })
        .collect();
    if let Some(error) = report.get("buildError") {
        let message = error.get("message").and_then(Value::as_str).unwrap_or("");
        let message = strip_parse_prefix(message);
        let line = first_line(error);
        problems.push(if is_image_lookup(message) {
            LineProblem::warning(line, message)
        } else {
            LineProblem::error(line, message)
        });
    }
    Ok(problems)
}

/// The 0-based start line of the first range in `location`.
fn first_line(entry: &Value) -> Option<usize> {
    entry
        .pointer("/location/ranges/0/start/line")
        .and_then(Value::as_u64)
        .and_then(|line| usize::try_from(line).ok())
        .map(|line| line.saturating_sub(1))
}

/// `dockerfile parse error on line 2: unknown instruction: RUNN` loses its
/// prefix; the line shows where.
fn strip_parse_prefix(message: &str) -> &str {
    message
        .strip_prefix("dockerfile parse error on line ")
        .and_then(|rest| rest.split_once(": "))
        .map_or(message, |(_, rest)| rest)
}

/// True for an error from resolving a `FROM` image, for example `pull access
/// denied` for an image that does not exist or needs a login.
fn is_image_lookup(message: &str) -> bool {
    message.contains("failed to resolve source metadata")
}

#[cfg(test)]
mod tests;
