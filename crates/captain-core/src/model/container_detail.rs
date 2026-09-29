use super::EnvVar;

/// The details the inspector shows for one container, from `inspect`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ContainerDetail {
    pub id: String,
    pub command: String,
    pub env: Vec<EnvVar>,
    pub mounts: Vec<Mount>,
    pub networks: Vec<String>,
    /// The restart policy name, for example `unless-stopped`. Empty means `no`.
    pub restart_policy: String,
    /// The health checks the engine keeps (at most five), oldest first.
    pub health_checks: Vec<HealthCheck>,
    /// RFC 3339 start time. Empty if the container never started.
    pub started_at: String,
}

/// A volume or bind mount.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mount {
    /// The volume name, or the host path for a bind mount.
    pub source: String,
    pub destination: String,
}

/// One health-check run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HealthCheck {
    pub passed: bool,
}
