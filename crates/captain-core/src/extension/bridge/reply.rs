//! Replies to the page: each becomes a script that Captain runs in the web view,
//! which settles the pending Promise or calls the stream handlers.

use serde_json::{Value, json};

/// One answer to a call. A plain call gets one `Resolve` or `Reject`. A streaming
/// exec gets `Output` lines, then `Exit`, or a `Reject`.
#[derive(Debug, Clone, PartialEq)]
pub enum BridgeEvent {
    Resolve(Value),
    Reject(Value),
    Output { stderr: bool, line: String },
    Exit(i32),
}

impl BridgeEvent {
    /// A rejection with `{ message }`, the shape of a JavaScript `Error`.
    pub fn error(message: impl Into<String>) -> Self {
        Self::Reject(json!({ "message": message.into() }))
    }

    /// True for the last event of a call.
    pub fn is_last(&self) -> bool {
        !matches!(self, Self::Output { .. })
    }

    /// The script that delivers this event for call `id`.
    pub fn script(&self, id: u64) -> String {
        let (function, value) = match self {
            Self::Resolve(value) => ("resolve", value.clone()),
            Self::Reject(value) => ("reject", value.clone()),
            Self::Output { stderr, line } => {
                let key = if *stderr { "stderr" } else { "stdout" };
                ("output", json!({ key: line }))
            }
            Self::Exit(code) => ("exit", json!(code)),
        };
        format!("window.__captainBridge&&window.__captainBridge.{function}({id},{value});")
    }
}

/// The result of a finished command, as `ExecResult`: resolved for exit code 0, else
/// rejected, as Docker Desktop does.
pub fn exec_result(cmd: &str, code: i32, stdout: String, stderr: String) -> BridgeEvent {
    let result = json!({ "cmd": cmd, "code": code, "stdout": stdout, "stderr": stderr });
    if code == 0 {
        BridgeEvent::Resolve(result)
    } else {
        BridgeEvent::Reject(result)
    }
}

/// The backend's HTTP answer. The body is parsed as JSON when it is JSON. A status
/// outside 2xx rejects with `{ name, message, statusCode }` (`ServiceError`).
pub fn service_result(status: u16, body: &str) -> BridgeEvent {
    let value = serde_json::from_str(body).unwrap_or_else(|_| Value::String(body.to_string()));
    if (200..300).contains(&status) {
        BridgeEvent::Resolve(value)
    } else {
        BridgeEvent::Reject(json!({
            "name": "ServiceError",
            "message": body,
            "statusCode": status,
        }))
    }
}

#[cfg(test)]
mod tests;
