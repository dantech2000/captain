//! The messages that the `ddClient` shim posts with `window.ipc.postMessage`, and
//! where Captain answers each one.

use std::collections::BTreeMap;

use serde_json::Value;

use super::navigate::NavigateIntent;
use super::options::{ListOptions, OpenDialogOptions};

/// One call from the page. `id` matches the reply to the pending Promise.
#[derive(Debug, Clone, PartialEq)]
pub struct BridgeCall {
    pub id: u64,
    pub request: BridgeRequest,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BridgeRequest {
    /// `extension.vm.service.*`: HTTP to the backend.
    Service(ServiceRequest),
    /// `extension.vm.cli.exec`, `extension.host.cli.exec`, or `docker.cli.exec`.
    Exec {
        scope: ExecScope,
        exec: ExecRequest,
    },
    ListContainers(ListOptions),
    ListImages(ListOptions),
    Toast {
        level: ToastLevel,
        message: String,
    },
    OpenDialog(OpenDialogOptions),
    OpenExternal(String),
    /// `desktopUI.navigate.*`: a page of Captain's main window. The window asks the
    /// engine to find a named object first, which answers with its full ID.
    Navigate(NavigateIntent),
    /// `close()` on a streaming exec: stops the call with this ID.
    Close(u64),
}

/// Where an exec runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecScope {
    /// `docker exec` in the backend container.
    Vm,
    /// One of the extension's own host binaries.
    Host,
    /// The `docker` CLI.
    Docker,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecRequest {
    pub cmd: String,
    pub args: Vec<String>,
    pub cwd: Option<String>,
    pub env: BTreeMap<String, String>,
    /// True for `{ stream: … }`: one reply per output line, then the exit code.
    pub stream: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceRequest {
    pub method: String,
    /// A path on the backend, for example `/hello`.
    pub path: String,
    pub headers: BTreeMap<String, String>,
    pub body: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastLevel {
    Success,
    Warning,
    Error,
}

/// Who answers a request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    /// The engine side: the backend, a process, or the Engine API.
    Engine,
    /// The extension's window: toasts, dialogs, and the browser.
    Window,
}

impl BridgeRequest {
    pub fn route(&self) -> Route {
        match self {
            Self::Service(_)
            | Self::Exec { .. }
            | Self::ListContainers(_)
            | Self::ListImages(_) => Route::Engine,
            Self::Toast { .. }
            | Self::OpenDialog(_)
            | Self::OpenExternal(_)
            | Self::Navigate(_)
            | Self::Close(_) => Route::Window,
        }
    }
}

/// A message Captain cannot answer. `id` is set when the message had one, so the
/// page's Promise can be rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeError {
    pub id: Option<u64>,
    pub message: String,
}

/// Parses one message from the page.
pub fn parse_call(message: &str) -> Result<BridgeCall, BridgeError> {
    let value: Value = serde_json::from_str(message).map_err(|error| BridgeError {
        id: None,
        message: format!("not a bridge message: {error}"),
    })?;
    let id = value.get("id").and_then(Value::as_u64);
    let fail = |message: String| BridgeError { id, message };
    let id = id.ok_or_else(|| fail("the message has no id".into()))?;
    let method = value.get("method").and_then(Value::as_str).unwrap_or("");
    let params = value.get("params").cloned().unwrap_or(Value::Null);
    let request = request(method, &params).map_err(fail)?;
    Ok(BridgeCall { id, request })
}

const NAVIGATE: &str = "desktopUI.navigate.";

fn request(method: &str, params: &Value) -> Result<BridgeRequest, String> {
    let exec = |scope| exec(params).map(|exec| BridgeRequest::Exec { scope, exec });
    match method {
        "extension.vm.service.request" => service(params).map(BridgeRequest::Service),
        "extension.vm.cli.exec" => exec(ExecScope::Vm),
        "extension.host.cli.exec" => exec(ExecScope::Host),
        "docker.cli.exec" => exec(ExecScope::Docker),
        "docker.listContainers" => ListOptions::parse(params).map(BridgeRequest::ListContainers),
        "docker.listImages" => ListOptions::parse(params).map(BridgeRequest::ListImages),
        "desktopUI.toast" => toast(params),
        "desktopUI.dialog.showOpenDialog" => {
            Ok(BridgeRequest::OpenDialog(OpenDialogOptions::parse(params)))
        }
        m if m.starts_with(NAVIGATE) => {
            NavigateIntent::parse(&m[NAVIGATE.len()..], params).map(BridgeRequest::Navigate)
        }
        "host.openExternal" => text(params, "url").map(BridgeRequest::OpenExternal),
        "exec.close" => params
            .get("target")
            .and_then(Value::as_u64)
            .map(BridgeRequest::Close)
            .ok_or_else(|| "exec.close needs a target".into()),
        _ => Err(format!("{method} is not supported by Captain")),
    }
}

fn text(params: &Value, key: &str) -> Result<String, String> {
    params
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| format!("\"{key}\" must be a string"))
}

fn strings(value: Option<&Value>) -> BTreeMap<String, String> {
    value
        .and_then(Value::as_object)
        .into_iter()
        .flatten()
        .filter_map(|(k, v)| Some((k.clone(), v.as_str()?.to_string())))
        .collect()
}

fn exec(params: &Value) -> Result<ExecRequest, String> {
    let args = match params.get("args") {
        None | Some(Value::Null) => Vec::new(),
        Some(Value::Array(args)) => args
            .iter()
            .map(|arg| arg.as_str().map(unquote).ok_or("\"args\" must be strings"))
            .collect::<Result<_, _>>()?,
        Some(_) => return Err("\"args\" must be an array".into()),
    };
    Ok(ExecRequest {
        cmd: text(params, "cmd")?,
        args,
        cwd: params
            .get("cwd")
            .and_then(Value::as_str)
            .map(str::to_string),
        env: strings(params.get("env")),
        stream: params.get("stream").and_then(Value::as_bool) == Some(true),
    })
}

/// Drops one pair of matching quotes around an argument. Docker Desktop runs these
/// commands through a shell, so extensions quote arguments like `"{{json .}}"`.
pub fn unquote(arg: &str) -> String {
    let bytes = arg.as_bytes();
    let quoted =
        bytes.len() >= 2 && matches!(bytes[0], b'"' | b'\'') && bytes[bytes.len() - 1] == bytes[0];
    if quoted {
        arg[1..arg.len() - 1].to_string()
    } else {
        arg.to_string()
    }
}

fn service(params: &Value) -> Result<ServiceRequest, String> {
    let method = text(params, "method")?.to_ascii_uppercase();
    if method.is_empty() || !method.bytes().all(|b| b.is_ascii_alphabetic()) {
        return Err(format!("\"{method}\" is not an HTTP method"));
    }
    let path = text(params, "url")?;
    if !path.starts_with('/') || path.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return Err(format!("\"{path}\" is not a path on the backend"));
    }
    let body = match params.get("data") {
        None | Some(Value::Null) => None,
        Some(Value::String(text)) => Some(text.clone()),
        Some(other) => Some(other.to_string()),
    };
    Ok(ServiceRequest {
        method,
        path,
        headers: strings(params.get("headers")),
        body,
    })
}

fn toast(params: &Value) -> Result<BridgeRequest, String> {
    let level = match params.get("level").and_then(Value::as_str) {
        Some("success") => ToastLevel::Success,
        Some("warning") => ToastLevel::Warning,
        Some("error") => ToastLevel::Error,
        _ => return Err("a toast needs a level: success, warning, or error".into()),
    };
    Ok(BridgeRequest::Toast {
        level,
        message: text(params, "message")?,
    })
}

#[cfg(test)]
mod tests;
