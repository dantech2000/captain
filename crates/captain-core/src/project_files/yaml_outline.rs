//! A small reader for block YAML that finds where each key sits, so a check's
//! key path (`services.web.ports`) maps to a line, and completion knows which
//! mapping the cursor is in. It reads indentation only; flow collections and
//! multi-line plain scalars stay opaque.

/// One key or list item: where it starts and its path from the root. A list item
/// adds its index, for example `services.web.ports.0`.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Entry {
    line: usize,
    col: usize,
    /// The length of the key's text on the line; `None` for a list item.
    key_len: Option<usize>,
    path: Vec<String>,
}

/// Where the cursor is, for completion and hover.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CursorContext {
    /// The path of the mapping or list item the cursor's line belongs to.
    pub parents: Vec<String>,
    /// The start of a key typed so far, when the cursor is in a key.
    pub prefix: Option<String>,
    /// The path of the key under the cursor, for hover.
    pub hovered: Option<Vec<String>>,
}

/// The 0-based line of the key at `path`, for example `["services", "web",
/// "ports"]`. A list item's path ends with its index.
pub fn key_line(text: &str, path: &[&str]) -> Option<usize> {
    outline(text)
        .into_iter()
        .find(|entry| {
            entry
                .path
                .iter()
                .map(String::as_str)
                .eq(path.iter().copied())
        })
        .map(|entry| entry.line)
}

/// What surrounds byte `offset` of `text`.
pub fn cursor_context(text: &str, offset: usize) -> CursorContext {
    let offset = floor_char_boundary(text, offset.min(text.len()));
    let line_start = text[..offset].rfind('\n').map_or(0, |at| at + 1);
    let line_no = text[..line_start].matches('\n').count();
    let line_end = text[offset..]
        .find('\n')
        .map_or(text.len(), |at| offset + at);
    let line = &text[line_start..line_end];
    let col = offset - line_start;
    let key_col = key_column(line);
    let entries = outline(text);

    let hovered = entries
        .iter()
        .filter(|entry| entry.line == line_no)
        .find(|entry| {
            entry
                .key_len
                .is_some_and(|len| entry.col <= col && col <= entry.col + len)
        })
        .map(|entry| entry.path.clone());
    let parents = entries
        .iter()
        .rev()
        .filter(|entry| entry.line <= line_no && entry.col < key_col)
        .find(|entry| entry.line < line_no || entry.key_len.is_none())
        .map(|entry| entry.path.clone())
        .unwrap_or_default();
    let typed = line.get(key_col..col).unwrap_or("");
    let prefix = (col >= key_col && typed.chars().all(is_key_char)).then(|| typed.to_string());
    CursorContext {
        parents,
        prefix,
        hovered,
    }
}

fn is_key_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.')
}

/// The column where a key on `line` starts: after the indentation and any `- `.
fn key_column(line: &str) -> usize {
    let mut col = indent(line);
    let mut rest = &line[col..];
    while rest == "-" || rest.starts_with("- ") {
        let skip = 1 + indent(&rest[1..]);
        col += skip;
        rest = &rest[skip..];
    }
    col
}

fn indent(line: &str) -> usize {
    line.len() - line.trim_start_matches(' ').len()
}

fn floor_char_boundary(text: &str, mut at: usize) -> usize {
    while !text.is_char_boundary(at) {
        at -= 1;
    }
    at
}

/// Every key and list item of `text`, in order.
fn outline(text: &str) -> Vec<Entry> {
    let mut entries = Vec::new();
    // Open keys and items: column, whether it is an item, and its path segment.
    let mut stack: Vec<(usize, bool, String)> = Vec::new();
    // Lines deeper than this column belong to a `|` or `>` block scalar.
    let mut block: Option<usize> = None;
    for (line_no, line) in text.lines().enumerate() {
        let mut col = indent(line);
        let mut rest = line[col..].trim_end();
        if rest.is_empty() || rest.starts_with('#') {
            continue;
        }
        if let Some(block_col) = block {
            if col > block_col {
                continue;
            }
            block = None;
        }
        if rest.starts_with("---") || rest.starts_with("...") {
            stack.clear();
            continue;
        }
        while rest == "-" || rest.starts_with("- ") {
            let index = close(&mut stack, col, true);
            stack.push((col, true, index.to_string()));
            entries.push(entry(line_no, col, None, &stack));
            let skip = 1 + indent(&rest[1..]);
            col += skip;
            rest = &rest[skip.min(rest.len())..];
        }
        if let Some((key, len, value)) = split_key(rest) {
            close(&mut stack, col, false);
            stack.push((col, false, key));
            entries.push(entry(line_no, col, Some(len), &stack));
            if value.starts_with('|') || value.starts_with('>') {
                block = Some(col);
            }
        }
    }
    entries
}

fn entry(
    line: usize,
    col: usize,
    key_len: Option<usize>,
    stack: &[(usize, bool, String)],
) -> Entry {
    Entry {
        line,
        col,
        key_len,
        path: stack.iter().map(|(_, _, seg)| seg.clone()).collect(),
    }
}

/// Closes the keys and items that a new one at `col` ends, and returns the index
/// for a new item. A key at the same column is the parent of an item there, as in
/// `ports:` followed by `- 80` without indentation.
fn close(stack: &mut Vec<(usize, bool, String)>, col: usize, item: bool) -> usize {
    let mut next = 0;
    while let Some((top_col, top_item, seg)) = stack.last() {
        let ends = *top_col > col || (*top_col == col && (!item || *top_item));
        if !ends {
            break;
        }
        if item && *top_item && *top_col == col {
            next = seg.parse::<usize>().map_or(0, |index| index + 1);
        }
        stack.pop();
    }
    next
}

/// Splits `key: value` into the key without quotes, the key's length on the
/// line, and the value. `None` if the line holds no key.
fn split_key(rest: &str) -> Option<(String, usize, &str)> {
    if let Some(quote) = rest.chars().next().filter(|c| matches!(c, '"' | '\'')) {
        let end = rest[1..].find(quote)? + 1;
        let value = rest[end + 1..].strip_prefix(':')?;
        return (value.is_empty() || value.starts_with(' '))
            .then(|| (rest[1..end].to_string(), end + 1, value.trim()));
    }
    if rest.starts_with(['{', '[', '#']) {
        return None;
    }
    let at = rest
        .find(": ")
        .or_else(|| rest.strip_suffix(':').map(str::len))?;
    let key = &rest[..at];
    (!key.is_empty()).then(|| (key.to_string(), at, rest[at + 1..].trim()))
}

#[cfg(test)]
mod tests;
