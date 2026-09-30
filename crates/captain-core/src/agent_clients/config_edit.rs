//! Adds and removes Captain's server in a client's JSON file in place, so the
//! user's comments, key order, and other servers stay. Remove after add gives the
//! file back as it was.

use jsonc_parser::cst::CstRootNode;
use serde_json::Value;

use super::client::SERVER_NAME;
use crate::settings::jsonc::{cst_value, options, parse};

/// `text` with `entry` as the `captain` server in the object at `key`. A blank
/// file becomes a new object. Text that is not a JSON object is an error, so a
/// file the user can still fix is never replaced.
pub fn with_server(text: &str, key: &str, entry: &Value) -> Result<String, String> {
    let blank = text.trim().is_empty();
    let root = open(if blank { "{}\n" } else { text })?;
    let servers = root.object_value_or_set().object_value_or_set(key);
    match servers.get(SERVER_NAME) {
        Some(prop) => prop.set_value(cst_value(entry)),
        None => {
            servers.append(SERVER_NAME, cst_value(entry));
        }
    }
    Ok(root.to_string())
}

/// `text` without the `captain` server in the object at `key`. The object goes too
/// when Captain `created` it and nothing is left in it, not even a comment.
pub fn without_server(text: &str, key: &str, created: bool) -> Result<String, String> {
    if text.trim().is_empty() {
        return Ok(text.to_string());
    }
    let root = open(text)?;
    let Some(object) = root.object_value() else {
        return Ok(text.to_string());
    };
    let Some(servers) = object.object_value(key) else {
        return Ok(text.to_string());
    };
    if let Some(prop) = servers.get(SERVER_NAME) {
        prop.remove();
    }
    let bare = servers
        .to_string()
        .chars()
        .all(|c| c.is_whitespace() || "{},".contains(c));
    if created
        && bare
        && let Some(prop) = object.get(key)
    {
        prop.remove();
    }
    Ok(root.to_string())
}

/// True if the object at `key` in `text` has a `captain` server.
pub fn has_server(text: &str, key: &str) -> Result<bool, String> {
    let value = parse(text).map_err(|problem| problem.to_string())?;
    Ok(value
        .get(key)
        .and_then(|servers| servers.get(SERVER_NAME))
        .is_some())
}

fn open(text: &str) -> Result<CstRootNode, String> {
    let value = parse(text).map_err(|problem| problem.to_string())?;
    if !value.is_object() {
        return Err("The file does not hold one JSON object.".into());
    }
    CstRootNode::parse(text, &options()).map_err(|error| error.to_string())
}

/// True if Codex's `config.toml` has a `[mcp_servers.captain]` table.
pub fn codex_has_server(text: &str) -> bool {
    text.lines().map(str::trim).any(|line| {
        matches!(
            line,
            "[mcp_servers.captain]" | "[mcp_servers.\"captain\"]" | "[mcp_servers.'captain']"
        )
    })
}

#[cfg(test)]
mod tests;
