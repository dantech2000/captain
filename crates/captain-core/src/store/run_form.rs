use thiserror::Error;

use super::{NameError, validate_name};
use crate::model::{EnvVar, ExposedPort, ImageDetail, PublishPort, RestartPolicy, RunSpec};
use crate::new_project::{ServiceSpec, named_volume};

/// The Run an image form, as the user typed it. [`RunForm::to_spec`] checks it for
/// a plain container, and [`RunForm::to_service`] for a Compose service.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RunForm {
    /// The image reference or ID to run.
    pub image: String,
    /// The container name. Empty lets the engine pick one.
    pub name: String,
    pub ports: Vec<PortRow>,
    /// `KEY=value` lines. Empty lines are skipped.
    pub env: Vec<String>,
    pub volumes: Vec<VolumeRow>,
    pub auto_remove: bool,
    pub restart: RestartPolicy,
}

/// One port row: the host port, and the container port such as `80` or `53/udp`.
/// An empty host port means "do not publish".
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PortRow {
    pub host: String,
    pub container: String,
}

/// One volume row: a volume name or a folder, and the path in the container.
/// A row with both fields empty is skipped.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VolumeRow {
    pub source: String,
    pub target: String,
}

/// Why the form's input is not valid.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RunFormError {
    #[error("Name: {0}")]
    Name(NameError),
    #[error("Host port for {0}: enter a number from 1 to 65535, or leave it empty")]
    Port(String),
    #[error("\"{0}\" is not a container port, such as 80 or 53/udp")]
    ContainerPort(String),
    #[error("Host port {0}/{1} is used twice")]
    DuplicatePort(u16, String),
    #[error("\"{0}\" is not KEY=value")]
    Env(String),
    #[error("Volume {0}: enter a volume name or a folder, and an absolute path in the container")]
    Volume(String),
    #[error("Volume {0}: a folder works only in a project; turn on Save as a project")]
    FolderNeedsProject(String),
    #[error("Auto-remove works only with the restart policy \"no\"")]
    AutoRemoveWithRestart,
}

impl RunForm {
    /// A form for `image`, prefilled from its details: each exposed port on the same
    /// host port. The image's environment stays in the image, so the form leaves
    /// it out.
    pub fn from_image(image: impl Into<String>, detail: &ImageDetail) -> Self {
        Self {
            image: image.into(),
            ports: detail
                .config
                .exposed_ports
                .iter()
                .map(|port| PortRow {
                    host: port.port.to_string(),
                    container: port.to_string(),
                })
                .collect(),
            ..Self::default()
        }
    }

    /// Checks the input for a plain container and builds the spec. Returns the
    /// first problem it finds.
    pub fn to_spec(&self) -> Result<RunSpec, RunFormError> {
        let name = self.name.trim();
        if !name.is_empty() {
            validate_name(name).map_err(RunFormError::Name)?;
        }
        // The engine refuses this pair, so say it before the request.
        if self.auto_remove && self.restart != RestartPolicy::No {
            return Err(RunFormError::AutoRemoveWithRestart);
        }
        let volumes = self.mounts()?;
        if let Some(folder) = volumes
            .iter()
            .find(|mount| named_volume(mount).is_none() && !mount.starts_with('/'))
        {
            return Err(RunFormError::FolderNeedsProject(folder.clone()));
        }
        Ok(RunSpec {
            image: self.image.clone(),
            name: (!name.is_empty()).then(|| name.to_string()),
            ports: self.published_ports()?,
            env: self.env_vars()?,
            volumes,
            auto_remove: self.auto_remove,
            restart: self.restart,
        })
    }

    /// Checks the input for a Compose service. A folder that is not absolute is
    /// relative to the project folder.
    pub fn to_service(&self) -> Result<ServiceSpec, RunFormError> {
        Ok(ServiceSpec {
            image: self.image.trim().to_string(),
            restart: (self.restart != RestartPolicy::No).then(|| self.restart.name().to_string()),
            environment: self
                .env_vars()?
                .into_iter()
                .map(|var| (var.key, Some(var.value)))
                .collect(),
            ports: self
                .published_ports()?
                .into_iter()
                .map(|port| match port.protocol.as_str() {
                    "tcp" => format!("{}:{}", port.host, port.container),
                    protocol => format!("{}:{}/{protocol}", port.host, port.container),
                })
                .collect(),
            volumes: self.mounts()?,
            ..ServiceSpec::default()
        })
    }

    fn env_vars(&self) -> Result<Vec<EnvVar>, RunFormError> {
        self.env
            .iter()
            .filter_map(|line| parse_env(line).transpose())
            .collect()
    }

    /// The volume rows as `source:target`. A folder like `data/db` gets `./`.
    fn mounts(&self) -> Result<Vec<String>, RunFormError> {
        let mut mounts = Vec::new();
        for row in &self.volumes {
            let (source, target) = (row.source.trim(), row.target.trim());
            if source.is_empty() && target.is_empty() {
                continue;
            }
            let shown = format!("{source}:{target}");
            if source.is_empty() || !target.starts_with('/') || source.contains(':') {
                return Err(RunFormError::Volume(shown));
            }
            let plain_name = named_volume(&shown).is_some();
            let relative = !plain_name && !source.starts_with(['/', '.', '~']);
            mounts.push(match relative {
                true => format!("./{shown}"),
                false => shown,
            });
        }
        Ok(mounts)
    }

    fn published_ports(&self) -> Result<Vec<PublishPort>, RunFormError> {
        let mut ports: Vec<PublishPort> = Vec::new();
        for row in &self.ports {
            let host = row.host.trim();
            let container_text = row.container.trim();
            if host.is_empty() && container_text.is_empty() {
                continue;
            }
            let container = ExposedPort::parse(container_text)
                .ok_or_else(|| RunFormError::ContainerPort(container_text.to_string()))?;
            if host.is_empty() {
                continue;
            }
            let host = host
                .parse::<u16>()
                .ok()
                .filter(|port| *port > 0)
                .ok_or_else(|| RunFormError::Port(container.to_string()))?;
            let protocol = container.protocol.clone();
            if ports
                .iter()
                .any(|p| p.host == host && p.protocol == protocol)
            {
                return Err(RunFormError::DuplicatePort(host, protocol));
            }
            ports.push(PublishPort {
                host,
                container: container.port,
                protocol,
            });
        }
        Ok(ports)
    }
}

/// Parses one `KEY=value` line. An empty line gives `None`.
fn parse_env(line: &str) -> Result<Option<EnvVar>, RunFormError> {
    let line = line.trim();
    if line.is_empty() {
        return Ok(None);
    }
    match line.split_once('=') {
        Some((key, _)) if !key.is_empty() && !key.contains(char::is_whitespace) => {
            Ok(Some(EnvVar::parse(line)))
        }
        _ => Err(RunFormError::Env(line.to_string())),
    }
}

#[cfg(test)]
mod tests;
