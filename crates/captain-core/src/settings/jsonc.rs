//! Reads `settings.json` as JSONC: JSON with `//` and `/* */` comments and
//! trailing commas. See ADR 0013.

use jsonc_parser::ast;
use jsonc_parser::{CollectOptions, ParseOptions, parse_to_ast};
use serde_json::Value;

use super::FileProblem;

/// Comments and trailing commas, as editors allow in JSONC. Nothing looser, so
/// the file stays valid for editors that read it as JSONC.
pub(super) fn options() -> ParseOptions {
    ParseOptions {
        allow_comments: true,
        allow_trailing_commas: true,
        allow_loose_object_property_names: false,
        allow_missing_commas: false,
        allow_single_quoted_strings: false,
        allow_hexadecimal_numbers: false,
        allow_unary_plus_numbers: false,
        allow_bare_decimal_point_numbers: false,
        allow_non_finite_numbers: false,
        allow_extended_string_escapes: false,
    }
}

/// The value in `text`. An empty file is an empty object.
pub(super) fn parse(text: &str) -> Result<Value, FileProblem> {
    let value: Option<Value> =
        jsonc_parser::parse_to_serde_value(text, &options()).map_err(|error| FileProblem {
            line: error.line_display(),
            key: None,
            message: error.kind().to_string(),
        })?;
    Ok(value.unwrap_or_else(|| Value::Object(Default::default())))
}

/// The value in `text`, which must be one object, as a settings file is. Bad JSON
/// or another value is an error.
pub(super) fn parse_object(text: &str) -> Result<Value, FileProblem> {
    let value = parse(text)?;
    if !value.is_object() {
        return Err(FileProblem {
            line: 1,
            key: None,
            message: "The file must hold one object, in { }".into(),
        });
    }
    Ok(value)
}

/// The line, from 1, of the property at `path`, such as `["kubernetes", "port"]`,
/// or of its deepest parent in the file.
pub(super) fn line_of(text: &str, path: &[String]) -> Option<usize> {
    let parsed = parse_to_ast(text, &CollectOptions::default(), &options()).ok()?;
    let mut value = parsed.value.as_ref()?;
    let mut start = None;
    for name in path {
        let ast::Value::Object(object) = value else {
            break;
        };
        let Some(prop) = object
            .properties
            .iter()
            .find(|prop| prop.name.as_str() == name)
        else {
            break;
        };
        start = Some(prop.range.start);
        value = &prop.value;
    }
    Some(text[..start?].matches('\n').count() + 1)
}
