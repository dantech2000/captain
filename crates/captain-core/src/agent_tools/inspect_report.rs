//! The `inspect` answer: a container's summary, never the raw `inspect` JSON.
//! Environment values that look secret are masked.

use schemars::JsonSchema;
use serde::Serialize;

use super::container_rows::{group_of, port_links};
use super::mask::{MASK, mask_secrets};
use super::untrusted::{clean, plain, wrap_untrusted};
use crate::format::bytes_label;
use crate::model::{Container, ContainerDetail, EnvVar};

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct InspectReport {
    pub name: String,
    /// The first 12 characters of the ID.
    pub id: String,
    pub image: String,
    pub state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health: Option<String>,
    /// The Compose project, or `k8s:<namespace>`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    /// The command it runs. Container-controlled text.
    pub command: String,
    /// Every port, as the Docker CLI shows it: `0.0.0.0:8080->80/tcp`.
    pub ports: Vec<String>,
    /// Published ports as links.
    pub urls: Vec<String>,
    /// `source -> destination (volume)` or `(bind)`.
    pub mounts: Vec<String>,
    pub networks: Vec<String>,
    /// For example `unless-stopped`; `no` when it never restarts.
    pub restart_policy: String,
    /// For example `512 MB`; absent without a limit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_limit: Option<String>,
    /// CPUs it may use; absent without a limit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_limit: Option<f64>,
    /// RFC 3339; absent if it never started.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    pub exit_code: i64,
    /// The kernel killed the last run for using too much memory.
    pub oom_killed: bool,
    pub restart_count: i64,
    /// The environment; secret-looking values are `[masked]`. Container-controlled
    /// text.
    pub env: Vec<EnvEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct EnvEntry {
    pub key: String,
    pub value: String,
}

pub fn inspect_report(container: &Container, detail: &ContainerDetail) -> InspectReport {
    InspectReport {
        name: container.display_name(),
        id: container.short_id().into(),
        image: container.image.clone(),
        state: container.state.label().into(),
        health: container.health.map(|h| h.label().into()),
        project: group_of(container),
        service: container.compose.service.clone(),
        command: mask_secrets(&clean(&detail.command)),
        ports: container.ports.iter().map(ToString::to_string).collect(),
        urls: port_links(container),
        mounts: detail
            .mounts
            .iter()
            .map(|m| {
                let kind = if m.volume { "volume" } else { "bind" };
                format!("{} -> {} ({kind})", m.source, m.destination)
            })
            .collect(),
        networks: detail.networks.clone(),
        restart_policy: match detail.restart_policy.as_str() {
            "" => "no".into(),
            policy => policy.into(),
        },
        memory_limit: (detail.memory_limit > 0).then(|| bytes_label(detail.memory_limit)),
        cpu_limit: (detail.nano_cpus > 0).then(|| detail.nano_cpus as f64 / 1e9),
        started_at: Some(detail.started_at.clone()).filter(|at| !at.is_empty()),
        exit_code: detail.exit_code,
        oom_killed: detail.oom_killed,
        restart_count: detail.restart_count,
        env: detail.env.iter().map(masked).collect(),
    }
}

/// `var` with its value masked when the key or the value looks secret.
fn masked(var: &EnvVar) -> EnvEntry {
    let value = if var.is_secret() && !var.value.is_empty() {
        MASK.to_string()
    } else {
        mask_secrets(&clean(&var.value))
    };
    EnvEntry {
        key: clean(&var.key),
        value,
    }
}

impl InspectReport {
    /// The summary as lines, with the command and environment between
    /// untrusted-output delimiters.
    pub fn text(&self) -> String {
        let mut lines = vec![format!(
            "{} ({}) · {} · {}",
            self.name, self.id, self.state, self.image
        )];
        let mut add = |label: &str, value: String| {
            if !value.is_empty() {
                lines.push(format!("{label}: {value}"));
            }
        };
        add("Ports", self.ports.join(", "));
        add("Mounts", self.mounts.join(", "));
        add("Networks", self.networks.join(", "));
        add("Restart policy", self.restart_policy.clone());
        add(
            "Memory limit",
            self.memory_limit.clone().unwrap_or_default(),
        );
        add(
            "Last exit",
            format!(
                "code {}{}, {} restarts",
                self.exit_code,
                if self.oom_killed {
                    ", out of memory"
                } else {
                    ""
                },
                self.restart_count
            ),
        );
        let mut untrusted = format!("command: {}", self.command);
        for entry in &self.env {
            untrusted.push_str(&format!("\n{}={}", entry.key, entry.value));
        }
        let mut lines: Vec<String> = lines.iter().map(|line| plain(line)).collect();
        lines.push(wrap_untrusted(&self.name, &untrusted));
        lines.join("\n")
    }
}

#[cfg(test)]
mod tests;
