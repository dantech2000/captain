//! The marked block that puts `~/.captain/bin` on `PATH` in a shell file.

/// The first line of Captain's block.
pub const START: &str = "# >>> captain >>>";
/// The last line of Captain's block.
pub const END: &str = "# <<< captain <<<";

/// `text` with `block` (from [`START`] to [`END`]). An existing block is replaced
/// where it is; else the block goes at the end, after a blank line, so it runs
/// after other PATH changes.
pub fn with_block(text: &str, block: &str) -> String {
    if let Some((before, after)) = split_block(text) {
        return format!("{before}{block}{after}");
    }
    let mut out = text.to_string();
    if !out.is_empty() {
        if !out.ends_with('\n') {
            out.push('\n');
        }
        out.push('\n');
    }
    out.push_str(block);
    out
}

/// `text` without Captain's block, and without the blank line [`with_block`]
/// put before it. `None` when there is no block.
pub fn without_block(text: &str) -> Option<String> {
    let (before, after) = split_block(text)?;
    let before = match after.is_empty() && before.ends_with("\n\n") {
        true => &before[..before.len() - 1],
        false => before,
    };
    Some(format!("{before}{after}"))
}

/// The text before the block's first line and after its last line (with its newline).
fn split_block(text: &str) -> Option<(&str, &str)> {
    let start = line_start(text, START)?;
    let end_line = start + line_start(&text[start..], END)?;
    let end = text[end_line..]
        .find('\n')
        .map_or(text.len(), |ix| end_line + ix + 1);
    Some((&text[..start], &text[end..]))
}

/// The offset of the first line that is exactly `line`, ignoring spaces around it.
fn line_start(text: &str, line: &str) -> Option<usize> {
    let mut offset = 0;
    for current in text.split_inclusive('\n') {
        if current.trim() == line {
            return Some(offset);
        }
        offset += current.len();
    }
    None
}

#[cfg(test)]
mod tests;
