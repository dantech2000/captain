use thiserror::Error;

use super::{NameError, validate_name};
use crate::model::{EnvVar, ExposedPort, ImageDetail, PublishPort, RestartPolicy, RunSpec};

/// The Run dialog's input, as the user typed it. [`RunForm::to_spec`] checks it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RunForm {
    /// The image reference or ID to run.
    pub image: String,
    /// The container name. Empty lets the engine pick one.
    pub name: String,
    pub ports: Vec<PortRow>,
    /// `KEY=value` lines. Empty lines are skipped.
    pub env: Vec<String>,
    pub auto_remove: bool,
    pub restart: RestartPolicy,
}

/// One exposed port and the host port the user typed for it. Empty means "do not
/// publish".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortRow {
    pub container: ExposedPort,
    pub host: String,
}

/// Why the Run dialog's input is not valid.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RunFormError {
    #[error("Name: {0}")]
    Name(NameError),
    #[error("Host port for {0}: enter a number from 1 to 65535, or leave it empty")]
    Port(String),
    #[error("Host port {0}/{1} is used twice")]
    DuplicatePort(u16, String),
    #[error("\"{0}\" is not KEY=value")]
    Env(String),
    #[error("Auto-remove works only with the restart policy \"no\"")]
    AutoRemoveWithRestart,
}

impl RunForm {
    /// A form for `image`, prefilled from its details: each exposed port on the same
    /// host port, and the image's environment.
    pub fn from_image(image: impl Into<String>, detail: &ImageDetail) -> Self {
        Self {
            image: image.into(),
            ports: detail
                .config
                .exposed_ports
                .iter()
                .map(|port| PortRow {
                    container: port.clone(),
                    host: port.port.to_string(),
                })
                .collect(),
            env: detail
                .config
                .env
                .iter()
                .map(|var| format!("{}={}", var.key, var.value))
                .collect(),
            ..Self::default()
        }
    }

    /// Checks the input and builds the spec. Returns the first problem it finds.
    pub fn to_spec(&self) -> Result<RunSpec, RunFormError> {
        let name = self.name.trim();
        if !name.is_empty() {
            validate_name(name).map_err(RunFormError::Name)?;
        }
        // The engine refuses this pair, so say it before the request.
        if self.auto_remove && self.restart != RestartPolicy::No {
            return Err(RunFormError::AutoRemoveWithRestart);
        }
        Ok(RunSpec {
            image: self.image.clone(),
            name: (!name.is_empty()).then(|| name.to_string()),
            ports: self.published_ports()?,
            env: self
                .env
                .iter()
                .filter_map(|line| parse_env(line).transpose())
                .collect::<Result<_, _>>()?,
            auto_remove: self.auto_remove,
            restart: self.restart,
        })
    }

    fn published_ports(&self) -> Result<Vec<PublishPort>, RunFormError> {
        let mut ports: Vec<PublishPort> = Vec::new();
        for row in &self.ports {
            let host = row.host.trim();
            if host.is_empty() {
                continue;
            }
            let host = host
                .parse::<u16>()
                .ok()
                .filter(|port| *port > 0)
                .ok_or_else(|| RunFormError::Port(row.container.to_string()))?;
            let protocol = row.container.protocol.clone();
            if ports
                .iter()
                .any(|p| p.host == host && p.protocol == protocol)
            {
                return Err(RunFormError::DuplicatePort(host, protocol));
            }
            ports.push(PublishPort {
                host,
                container: row.container.port,
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
