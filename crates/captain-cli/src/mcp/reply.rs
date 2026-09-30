//! Tool results: `structuredContent` that matches the tool's `outputSchema`, with a
//! text copy for clients that show only text. Failures and refusals are results with
//! `isError`, not protocol errors, so the agent reads why.

use rmcp::ErrorData;
use rmcp::model::{CallToolResult, ContentBlock};
use serde::Serialize;

pub type ToolResult = Result<CallToolResult, ErrorData>;

pub fn reply<T: Serialize>(value: &T, text: String) -> ToolResult {
    let structured = serde_json::to_value(value)
        .map_err(|error| ErrorData::internal_error(error.to_string(), None))?;
    let mut result = CallToolResult::success(vec![ContentBlock::text(text)]);
    result.structured_content = Some(structured);
    Ok(result)
}

pub fn refuse(why: impl Into<String>) -> ToolResult {
    Ok(CallToolResult::error(vec![ContentBlock::text(why)]))
}
