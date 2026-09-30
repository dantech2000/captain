use super::{LineProblem, key_line};

/// Reads the error output of `docker compose -f - config --quiet`, run on the
/// unsaved `text` of one Compose file, as problems on the lines of `text`.
///
/// Schema errors name a key path (`validating -: services.web additional
/// properties 'imgae' not allowed`), which maps to the key's line; YAML errors
/// name lines (`at L2.C3-L4.C4: did not find expected key`). An error in another
/// file of the project has no line here.
pub fn compose_problems(stderr: &str, text: &str) -> Vec<LineProblem> {
    stderr
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .filter_map(|line| problem(line, text))
        .collect()
}

fn problem(line: &str, text: &str) -> Option<LineProblem> {
    if line.contains("level=warning") {
        let message = quoted_msg(line)?;
        let message = message.strip_prefix("-: ").unwrap_or(&message).to_string();
        let at = backticked(&message).and_then(|key| key_line(text, &[key]));
        return Some(LineProblem::warning(at, message));
    }
    if let Some(rest) = line.strip_prefix("validating ") {
        let (file, message) = rest.split_once(": ")?;
        if file != "-" {
            let name = file.rsplit(['/', '\\']).next().unwrap_or(file);
            return Some(LineProblem::error(None, format!("{name}: {message}")));
        }
        return Some(schema_error(message.trim_start(), text));
    }
    if let Some((_, rest)) = line.split_once("failed to parse -: ") {
        return Some(LineProblem::error(yaml_error_line(rest, text), rest));
    }
    if let Some(rest) = line.strip_prefix("error while interpolating ") {
        let at = rest
            .split_once(':')
            .and_then(|(path, _)| path_line(text, path));
        return Some(LineProblem::error(at, line));
    }
    let at = quoted_service(line).and_then(|name| key_line(text, &["services", name]));
    Some(LineProblem::error(at, line))
}

/// `services.web additional properties 'imgae' not allowed` points at `imgae`;
/// `services.web.ports must be a array` points at `ports`. The root has an empty
/// path.
fn schema_error(message: &str, text: &str) -> LineProblem {
    let (path, detail) = message.split_once(' ').unwrap_or((message, ""));
    let (path, detail) = if path == "additional" {
        ("", message)
    } else {
        (path, detail)
    };
    let extra = detail
        .strip_prefix("additional properties '")
        .and_then(|rest| rest.split_once('\''))
        .map(|(key, _)| key);
    let line = match extra {
        Some(key) if path.is_empty() => key_line(text, &[key]),
        Some(key) => path_line(text, &format!("{path}.{key}")),
        None => path_line(text, path),
    };
    LineProblem::error(line, message)
}

/// The line of the longest part of the dotted `path` that the text has, so a key
/// that is missing still points at its parent.
fn path_line(text: &str, path: &str) -> Option<usize> {
    let segments: Vec<&str> = path.split('.').filter(|s| !s.is_empty()).collect();
    (1..=segments.len())
        .rev()
        .find_map(|len| key_line(text, &segments[..len]))
}

/// The line of `L2.C3-L4.C4`: the end, where the parser gave up, unless that is
/// past the text or blank; then the start.
fn yaml_error_line(message: &str, text: &str) -> Option<usize> {
    let positions: Vec<usize> = message
        .split(|c: char| !c.is_ascii_alphanumeric() && c != '.')
        .filter_map(|word| word.strip_prefix('L')?.split_once(".C")?.0.parse().ok())
        .collect();
    let lines: Vec<&str> = text.lines().collect();
    let usable = |line: usize| lines.get(line).is_some_and(|l| !l.trim().is_empty());
    let (start, end) = match positions.as_slice() {
        [start, end, ..] => (*start, *end),
        [only] => (*only, *only),
        [] => return legacy_yaml_line(message),
    };
    let (start, end) = (start.saturating_sub(1), end.saturating_sub(1));
    Some(if usable(end) { end } else { start })
}

/// Older Compose versions say `yaml: line 4: did not find expected key`.
fn legacy_yaml_line(message: &str) -> Option<usize> {
    let rest = message.split_once("line ")?.1;
    let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
    digits
        .parse::<usize>()
        .ok()
        .map(|line| line.saturating_sub(1))
}

/// The `msg="..."` part of a logrus line, unescaped.
fn quoted_msg(line: &str) -> Option<String> {
    let rest = line.split_once("msg=\"")?.1;
    let end = rest.rfind('"')?;
    Some(rest[..end].replace("\\\"", "\""))
}

fn backticked(message: &str) -> Option<&str> {
    let rest = message.split_once('`')?.1;
    Some(rest.split_once('`')?.0)
}

/// `web` in `service "web" depends on undefined service "db"`.
fn quoted_service(line: &str) -> Option<&str> {
    let rest = line.split_once("service \"")?.1;
    Some(rest.split_once('"')?.0)
}

#[cfg(test)]
mod tests;
