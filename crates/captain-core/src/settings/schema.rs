//! The JSON Schema of `settings.json`, derived from [`Settings`] with `schemars`
//! (https://docs.rs/schemars/1.2.2/schemars/). The doc comments on the fields are
//! the descriptions. See ADR 0013.

use std::path::Path;

use schemars::generate::SchemaSettings;
use serde_json::Value;

use super::Settings;

/// The file name that the app writes next to `settings.json`, and that a new
/// file's `$schema` line names.
pub const SCHEMA_FILE: &str = "settings.schema.json";

/// The schema as JSON. Draft 7, because editors support it best.
pub fn settings_schema() -> Value {
    let schema = SchemaSettings::draft07()
        .into_generator()
        .into_root_schema_for::<Settings>();
    let mut value = serde_json::to_value(schema).unwrap_or_default();
    unwrap_descriptions(&mut value);
    if let Some(root) = value.as_object_mut() {
        // VS Code reads these, and then accepts comments and trailing commas.
        root.insert("allowComments".into(), Value::Bool(true));
        root.insert("allowTrailingCommas".into(), Value::Bool(true));
    }
    value
}

/// Joins the lines of each description, which come from wrapped doc comments, and
/// keeps blank lines between paragraphs.
fn unwrap_descriptions(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for (key, inner) in map.iter_mut() {
                match inner {
                    Value::String(text) if key == "description" => {
                        *text = text
                            .split("\n\n")
                            .map(|paragraph| paragraph.replace('\n', " "))
                            .collect::<Vec<_>>()
                            .join("\n\n");
                    }
                    _ => unwrap_descriptions(inner),
                }
            }
        }
        Value::Array(items) => items.iter_mut().for_each(unwrap_descriptions),
        _ => {}
    }
}

/// Writes `settings.schema.json` next to `settings_path`, unless it is already
/// up to date.
pub fn write_schema(settings_path: &Path) -> std::io::Result<()> {
    let path = settings_path.with_file_name(SCHEMA_FILE);
    let text = settings_schema_text();
    if std::fs::read_to_string(&path).is_ok_and(|old| old == text) {
        return Ok(());
    }
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(path, text)
}

/// The text of `settings.schema.json`.
pub fn settings_schema_text() -> String {
    let mut text = serde_json::to_string_pretty(&settings_schema()).unwrap_or_default();
    text.push('\n');
    text
}
