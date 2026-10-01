//! The Compose files Captain writes for Run an image and Paste a docker run
//! command: one service, written as plain YAML that a person can read and edit.

use std::fmt::Write as _;

/// One Compose service. Strings are literal values: [`ComposeDoc::to_yaml`]
/// quotes them and escapes `$`, so Compose does not interpolate them.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ServiceSpec {
    pub image: String,
    pub container_name: Option<String>,
    pub hostname: Option<String>,
    pub platform: Option<String>,
    pub pull_policy: Option<String>,
    /// `no` is left out, as Compose's default.
    pub restart: Option<String>,
    pub entrypoint: Vec<String>,
    pub command: Vec<String>,
    pub user: Option<String>,
    pub working_dir: Option<String>,
    /// A `None` value passes the variable from the shell that runs Compose.
    pub environment: Vec<(String, Option<String>)>,
    pub env_file: Vec<String>,
    /// Short syntax, for example `8080:80` or `127.0.0.1:53:53/udp`.
    pub ports: Vec<String>,
    /// Short syntax, for example `data:/var/lib/data` or `./site:/srv:ro`.
    pub volumes: Vec<String>,
    /// Networks that exist already; they are declared `external: true`.
    pub networks: Vec<String>,
    pub network_mode: Option<String>,
    pub labels: Vec<(String, String)>,
    pub stdin_open: bool,
    pub tty: bool,
    pub mem_limit: Option<String>,
    pub cpus: Option<String>,
}

/// A Compose file with its services. Named volumes and external networks are
/// declared at the top level from what the services use.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ComposeDoc {
    /// The top-level `name:`, the Compose project name.
    pub name: Option<String>,
    pub services: Vec<(String, ServiceSpec)>,
}

impl ComposeDoc {
    /// The file's text.
    pub fn to_yaml(&self) -> String {
        let mut out = String::new();
        if let Some(name) = &self.name {
            line(&mut out, 0, &format!("name: {}", scalar(name)));
        }
        line(&mut out, 0, "services:");
        for (name, service) in &self.services {
            line(&mut out, 1, &format!("{}:", key(name)));
            write_service(&mut out, service);
        }
        let volumes = self.named_volumes();
        if !volumes.is_empty() {
            line(&mut out, 0, "volumes:");
            for volume in volumes {
                line(&mut out, 1, &format!("{}: {{}}", key(&volume)));
            }
        }
        let networks = self.networks();
        if !networks.is_empty() {
            line(&mut out, 0, "networks:");
            for network in networks {
                line(&mut out, 1, &format!("{}:", key(&network)));
                line(&mut out, 2, "external: true");
            }
        }
        out
    }

    /// The named volumes the services mount, in order, once each.
    pub fn named_volumes(&self) -> Vec<String> {
        let mut names: Vec<String> = Vec::new();
        for (_, service) in &self.services {
            for name in service.volumes.iter().filter_map(|v| named_volume(v)) {
                if !names.iter().any(|known| known == name) {
                    names.push(name.to_string());
                }
            }
        }
        names
    }

    fn networks(&self) -> Vec<String> {
        let mut names: Vec<String> = Vec::new();
        for (_, service) in &self.services {
            for name in &service.networks {
                if !names.contains(name) {
                    names.push(name.clone());
                }
            }
        }
        names
    }
}

/// The volume name of a short-syntax mount, or `None` for a folder or an
/// anonymous volume. A name starts with a letter or a digit and has no `/`.
pub fn named_volume(mount: &str) -> Option<&str> {
    let (source, _) = mount.split_once(':')?;
    let mut chars = source.chars();
    let first = chars.next()?;
    let named = first.is_ascii_alphanumeric()
        && source
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-'))
        // `C:\data:/data` is a Windows folder.
        && !(source.len() == 1 && mount[2..].starts_with(['\\', '/']));
    named.then_some(source)
}

fn write_service(out: &mut String, s: &ServiceSpec) {
    let single = |out: &mut String, name: &str, value: &Option<String>| {
        if let Some(value) = value {
            line(out, 2, &format!("{name}: {}", scalar(value)));
        }
    };
    line(out, 2, &format!("image: {}", scalar(&s.image)));
    single(out, "container_name", &s.container_name);
    single(out, "hostname", &s.hostname);
    single(out, "platform", &s.platform);
    single(out, "pull_policy", &s.pull_policy);
    single(out, "restart", &s.restart);
    flow_list(out, "entrypoint", &s.entrypoint);
    flow_list(out, "command", &s.command);
    single(out, "user", &s.user);
    single(out, "working_dir", &s.working_dir);
    if !s.environment.is_empty() {
        line(out, 2, "environment:");
        for (name, value) in &s.environment {
            match value {
                Some(value) => line(out, 3, &format!("{}: {}", key(name), scalar(value))),
                None => line(out, 3, &format!("{}:", key(name))),
            }
        }
    }
    block_list(out, "env_file", &s.env_file);
    block_list(out, "ports", &s.ports);
    block_list(out, "volumes", &s.volumes);
    block_list(out, "networks", &s.networks);
    single(out, "network_mode", &s.network_mode);
    if !s.labels.is_empty() {
        line(out, 2, "labels:");
        for (name, value) in &s.labels {
            line(out, 3, &format!("{}: {}", key(name), scalar(value)));
        }
    }
    if s.stdin_open {
        line(out, 2, "stdin_open: true");
    }
    if s.tty {
        line(out, 2, "tty: true");
    }
    single(out, "mem_limit", &s.mem_limit);
    single(out, "cpus", &s.cpus);
}

fn line(out: &mut String, depth: usize, text: &str) {
    let _ = writeln!(out, "{:width$}{text}", "", width = depth * 2);
}

fn block_list(out: &mut String, name: &str, items: &[String]) {
    if items.is_empty() {
        return;
    }
    line(out, 2, &format!("{name}:"));
    for item in items {
        line(out, 3, &format!("- {}", scalar(item)));
    }
}

/// `name: ["a", "b"]`, each item quoted.
fn flow_list(out: &mut String, name: &str, items: &[String]) {
    if items.is_empty() {
        return;
    }
    let items: Vec<String> = items.iter().map(|item| quoted(item)).collect();
    line(out, 2, &format!("{name}: [{}]", items.join(", ")));
}

/// A mapping key: plain when it is a simple word, else quoted.
fn key(text: &str) -> String {
    if is_plain(text) {
        text.to_string()
    } else {
        serde_json::to_string(text).expect("a string serializes")
    }
}

/// A value: plain when YAML reads it back as the same string, else quoted.
/// `$` is doubled either way, so Compose keeps it.
fn scalar(text: &str) -> String {
    let text = text.replace('$', "$$");
    if is_plain(&text) {
        text
    } else {
        serde_json::to_string(&text).expect("a string serializes")
    }
}

fn quoted(text: &str) -> String {
    serde_json::to_string(&text.replace('$', "$$")).expect("a string serializes")
}

/// True when `text` is a plain YAML string that no parser reads as a number, a
/// bool, a null, or a time: a letter, `/`, `./`, or `../` first, then letters,
/// digits, and `_ . / - : @`, without a final `:`.
fn is_plain(text: &str) -> bool {
    const WORDS: [&str; 11] = [
        "true", "false", "yes", "no", "on", "off", "null", "y", "n", "nan", "inf",
    ];
    let Some(first) = text.chars().next() else {
        return false;
    };
    (first.is_ascii_alphabetic()
        || first == '/'
        || text.starts_with("./")
        || text.starts_with("../"))
        && text
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '/' | '-' | ':' | '@'))
        && !text.ends_with(':')
        && !text.contains("::")
        && !WORDS.contains(&text.to_ascii_lowercase().as_str())
}

#[cfg(test)]
mod tests;
