use crate::model::EnvVar;

/// What `docker run -d` needs to create and start a container from an image.
/// Build it with [`RunForm`](crate::store::RunForm), which checks the user's input.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RunSpec {
    /// The image reference or ID.
    pub image: String,
    /// The container name. `None` lets the engine pick one.
    pub name: Option<String>,
    pub ports: Vec<PublishPort>,
    pub env: Vec<EnvVar>,
    /// Remove the container when it exits, like `--rm`.
    pub auto_remove: bool,
    pub restart: RestartPolicy,
}

/// One published port: the host port that forwards to a container port.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishPort {
    pub host: u16,
    pub container: u16,
    /// `tcp`, `udp`, or `sctp`.
    pub protocol: String,
}

impl PublishPort {
    /// The container side as the engine keys it, for example `80/tcp`.
    pub fn container_key(&self) -> String {
        format!("{}/{}", self.container, self.protocol)
    }
}

/// When the engine restarts a container.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum RestartPolicy {
    #[default]
    No,
    UnlessStopped,
    Always,
    OnFailure,
}

impl RestartPolicy {
    pub const ALL: [RestartPolicy; 4] = [
        RestartPolicy::No,
        RestartPolicy::UnlessStopped,
        RestartPolicy::Always,
        RestartPolicy::OnFailure,
    ];

    /// Parses the name `inspect` reports. Empty and unknown names mean `no`.
    pub fn parse(name: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|policy| policy.name() == name)
            .unwrap_or_default()
    }

    /// The name the engine and the CLI use, for example `unless-stopped`.
    pub fn name(self) -> &'static str {
        match self {
            RestartPolicy::No => "no",
            RestartPolicy::UnlessStopped => "unless-stopped",
            RestartPolicy::Always => "always",
            RestartPolicy::OnFailure => "on-failure",
        }
    }
}
