//! docs/reference/settings.md, generated from [`reference_entries`]. A test fails
//! when the committed file differs; see `reference_markdown/tests.rs`.

use std::fmt::Write;

use super::{GROUPS, ReferenceEntry, SCHEMA_FILE, reference_entries};

const INTRO: &str = "\
# Settings reference

<!-- Generated from the settings types in crates/captain-core. Do not edit by hand.
     Run `CAPTAIN_BLESS=1 cargo test -p captain-core reference` after a change. -->

Captain keeps its settings in `settings.json`:

- macOS: `~/Library/Application Support/Captain/settings.json`
- Linux: `~/.config/Captain/settings.json`
- Windows: `%APPDATA%\\Captain\\settings.json`

The file holds only the settings you change. Every other key has the default listed \
here, so a new default in a later version reaches you. The file may have `//` and \
`/* */` comments and trailing commas. When Captain or `captain set` changes a value, \
it edits that value in place, so your comments stay.

Captain writes `SCHEMA` next to the file. The `\"$schema\": \"./SCHEMA\"` line in the \
file gives editors such as Zed and VS Code completion, descriptions, and checks. \
`version` is the file format; Captain writes it.

Captain watches the file and applies a change when you save it. If a value is wrong, \
Captain keeps the last good settings and names the line, for example: \
`settings.json line 9: kubernetes.port must be a whole number from 1 to 65535.`
";

/// The text of docs/reference/settings.md.
pub fn reference_markdown() -> String {
    let mut text = INTRO.replace("SCHEMA", SCHEMA_FILE);
    let entries = reference_entries();
    for group in GROUPS {
        let _ = write!(text, "\n## {group}\n");
        for entry in entries.iter().filter(|entry| entry.group == group) {
            section(&mut text, entry);
        }
    }
    text
}

fn section(text: &mut String, entry: &ReferenceEntry) {
    let _ = write!(
        text,
        "\n### `{key}`\n\n{description}\n\n- Type: {kind}\n- Default: `{default}`\n",
        key = entry.key,
        description = entry.description,
        kind = entry.type_label,
        default = entry.default,
    );
    if let Some(example) = &entry.example {
        let _ = writeln!(text, "- Example: `{}`", snippet(&entry.key, example));
    }
}

/// `"theme": "harbor"`, or `"kubernetes": { "port": 16443 }` for a key in a group.
fn snippet(key: &str, example: &str) -> String {
    match key.split_once('.') {
        Some((group, inner)) => format!("\"{group}\": {{ \"{inner}\": {example} }}"),
        None => format!("\"{key}\": {example}"),
    }
}

#[cfg(test)]
mod tests;
