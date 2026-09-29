//! The `limactl` arguments for each action, and the progress lines it prints.

use std::path::Path;

use captain_core::{GIB, HostResources};

use super::template::{disk_size, memory_size};

/// How long `limactl start` waits for the engine. The first start downloads and
/// installs Docker and can be slow (lima-vm/lima#4517), so this is generous.
pub const START_TIMEOUT: &str = "30m";

/// Log lines as JSON, so progress shows only the message.
fn base(command: &str) -> Vec<String> {
    ["--log-format", "json", command].map(String::from).to_vec()
}

fn with(mut args: Vec<String>, rest: &[&str]) -> Vec<String> {
    args.extend(rest.iter().map(ToString::to_string));
    args
}

pub fn list() -> Vec<String> {
    vec!["list".into(), "--json".into()]
}

pub fn create(name: &str, template: &Path) -> Vec<String> {
    let mut args = with(base("create"), &["--name", name, "--tty=false"]);
    args.push(template.display().to_string());
    args
}

pub fn start(name: &str) -> Vec<String> {
    with(
        base("start"),
        &[
            name,
            "--tty=false",
            "--progress",
            "--timeout",
            START_TIMEOUT,
        ],
    )
}

pub fn stop(name: &str, force: bool) -> Vec<String> {
    let args = with(base("stop"), &[name, "--tty=false"]);
    if force {
        with(args, &["--force"])
    } else {
        args
    }
}

pub fn delete(name: &str) -> Vec<String> {
    with(base("delete"), &[name, "--force", "--tty=false"])
}

/// `limactl edit` flags that change `current` into `wanted`, or `None` when nothing
/// changes. A disk can only grow, so a smaller disk is left as it is.
pub fn edit(name: &str, current: &HostResources, wanted: &HostResources) -> Option<Vec<String>> {
    let mut args = with(base("edit"), &[name, "--tty=false"]);
    let before = args.len();
    if current.cpus != wanted.cpus {
        args.extend(["--cpus".into(), wanted.cpus.to_string()]);
    }
    if current.memory_bytes != wanted.memory_bytes {
        let mib = memory_size(wanted.memory_bytes);
        let gib = mib.trim_end_matches("MiB").parse::<f64>().unwrap_or(1024.0) / 1024.0;
        args.extend(["--memory".into(), format_gib(gib)]);
    }
    if wanted.disk_bytes / GIB > current.disk_bytes / GIB {
        let disk = disk_size(wanted.disk_bytes);
        args.extend(["--disk".into(), disk.trim_end_matches("GiB").to_string()]);
    }
    (args.len() > before).then_some(args)
}

/// `8` or `4.5`: `limactl edit` takes GiB as a decimal number.
fn format_gib(gib: f64) -> String {
    let text = format!("{gib:.3}");
    text.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// Messages that mean nothing to a Captain user.
const NOISE: &[&str] = &["Terminal is not available"];

/// The text to show for one output line of `limactl`, or `None` to skip it. JSON log
/// lines give their message; debug lines are skipped; other lines show as they are.
pub fn progress_line(raw: &str) -> Option<String> {
    let line = raw.trim();
    if line.is_empty() {
        return None;
    }
    let Ok(serde_json::Value::Object(log)) = serde_json::from_str::<serde_json::Value>(line) else {
        return Some(line.to_string());
    };
    let level = log.get("level").and_then(|v| v.as_str()).unwrap_or("info");
    if matches!(level, "debug" | "trace") {
        return None;
    }
    let message = log.get("msg").and_then(|v| v.as_str())?.trim();
    if message.is_empty() || NOISE.iter().any(|noise| message.starts_with(noise)) {
        return None;
    }
    match level {
        "warning" | "error" | "fatal" => Some(format!("{level}: {message}")),
        _ => Some(message.to_string()),
    }
}

#[cfg(test)]
mod tests;
