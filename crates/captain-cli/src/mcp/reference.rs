//! The Markdown reference of the MCP tools, built from the server's own tool list:
//! the descriptions, arguments, and annotations that agents read. `captain docs
//! mcp` prints it into docs/reference/mcp.md, and a test checks the committed file.

use std::fmt::Write as _;
use std::sync::Arc;

use captain_core::agent_tools::AgentAction;
use rmcp::model::Tool;
use serde_json::Value;

use super::{CaptainServer, Connect, Source};

/// The regeneration command, printed in the page so readers know not to edit it.
pub const REGENERATE: &str = "cargo run -p captain-cli -- docs mcp > docs/reference/mcp.md";

/// The whole page.
pub fn markdown() -> String {
    let connect: Connect = Arc::new(|| Err("no engine".into()));
    let server = CaptainServer::new(Source::new("Other engine", None, connect, false));
    let tools = server.all_tools();
    let mut out = String::from("# captain mcp tool reference\n\n");
    let _ = writeln!(
        out,
        "<!-- Generated from the tool definitions in crates/captain-cli/src/mcp. Do not \
         edit. Run `{REGENERATE}` after a change to the tools. -->\n"
    );
    out.push_str(
        "`captain mcp` is Captain's MCP server for AI agents, over stdio. This page lists \
         every tool, with the description and arguments that agents read. \
         [AI agents](../guide/agents.md) in the user guide explains how to connect one.\n\n\
         A tool is listed only while the settings allow it: `help` always, the read tools \
         while `agent_tools.enabled` is `true`, and each action while `agent_tools.actions` \
         also names it. See [the settings reference](settings.md#ai-agents).\n\n\
         A call with an argument that a tool does not take is refused with `isError`, \
         and the message lists the arguments it takes.\n\n",
    );
    for (title, read_only) in [("Read tools", true), ("Actions", false)] {
        let _ = writeln!(out, "## {title}\n");
        for tool in tools.iter().filter(|tool| is_read_only(tool) == read_only) {
            section(&mut out, tool);
        }
    }
    out.trim_end().to_string() + "\n"
}

fn is_read_only(tool: &Tool) -> bool {
    tool.annotations
        .as_ref()
        .and_then(|annotations| annotations.read_only_hint)
        .unwrap_or(false)
}

fn section(out: &mut String, tool: &Tool) {
    let _ = writeln!(out, "### `{}`\n", tool.name);
    if let Some(description) = &tool.description {
        let _ = writeln!(
            out,
            "{}\n",
            description.split_whitespace().collect::<Vec<_>>().join(" ")
        );
    }
    let _ = writeln!(out, "- Listed when: {}", allowed_by(&tool.name));
    if let Some(title) = tool.annotations.as_ref().and_then(|a| a.title.as_deref()) {
        let _ = writeln!(out, "- Title: {title}");
    }
    let properties = tool
        .input_schema
        .get("properties")
        .and_then(Value::as_object);
    let required: Vec<&str> = tool
        .input_schema
        .get("required")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect();
    match properties.filter(|properties| !properties.is_empty()) {
        None => out.push_str("- Arguments: none\n"),
        Some(properties) => {
            out.push_str("- Arguments:\n");
            for (name, schema) in properties {
                let need = if required.contains(&name.as_str()) {
                    "required"
                } else {
                    "optional"
                };
                let text = schema
                    .get("description")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ");
                let _ = writeln!(out, "  - `{name}` ({}, {need}): {text}", kind(schema));
            }
        }
    }
    out.push('\n');
}

/// The setting that lists the tool.
fn allowed_by(tool: &str) -> String {
    match AgentAction::of_tool(tool) {
        _ if tool == "help" => "always".into(),
        Some(action) => format!(
            "`agent_tools.enabled` is `true` and `agent_tools.actions` includes `\"{}\"`",
            action.name()
        ),
        None => "`agent_tools.enabled` is `true`".into(),
    }
}

/// The JSON type of an argument, without `null`.
fn kind(schema: &Value) -> String {
    let kinds: Vec<&str> = match schema.get("type") {
        Some(Value::String(one)) => vec![one],
        Some(Value::Array(many)) => many.iter().filter_map(Value::as_str).collect(),
        _ => Vec::new(),
    };
    kinds
        .into_iter()
        .find(|kind| *kind != "null")
        .unwrap_or("any")
        .to_string()
}

#[cfg(test)]
mod tests;
