//! The init script that defines `window.ddClient` before the extension's page loads.

use serde_json::json;

const SHIM: &str = include_str!("shim.js");

/// The plain values the shim needs about the extension and the host.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShimContext {
    pub id: String,
    pub image: String,
    /// `darwin`, `linux`, or `win32`, as Node's `process.platform`.
    pub platform: String,
    /// `arm64` or `x64`, as Node's `process.arch`.
    pub arch: String,
    pub hostname: String,
}

impl ShimContext {
    /// The context for this computer.
    pub fn for_host(id: String, image: String, hostname: String) -> Self {
        let platform = match std::env::consts::OS {
            "macos" => "darwin",
            "windows" => "win32",
            other => other,
        };
        let arch = match std::env::consts::ARCH {
            "aarch64" => "arm64",
            "x86_64" => "x64",
            other => other,
        };
        Self {
            id,
            image,
            platform: platform.into(),
            arch: arch.into(),
            hostname,
        }
    }
}

/// The shim with `context` filled in.
pub fn init_script(context: &ShimContext) -> String {
    let values = json!({
        "id": context.id,
        "image": context.image,
        "platform": context.platform,
        "arch": context.arch,
        "hostname": context.hostname,
    });
    SHIM.replace("__CAPTAIN_CONTEXT__", &values.to_string())
}

#[cfg(test)]
mod tests;
