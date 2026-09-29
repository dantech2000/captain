//! The Docker implementation of the UI's [`MigrationBackend`].

use std::path::Path;
use std::sync::Arc;

use captain_core::EngineError;
use captain_core::migration::{MigrationBackend, MigrationSession, SourceOption};

use super::DockerSession;
use crate::{CandidateSource, DiscoveryInput, Endpoint, candidates};

/// Finds engines on this machine and opens [`DockerSession`]s.
#[derive(Debug, Clone, Copy, Default)]
pub struct DockerMigrator;

impl MigrationBackend for DockerMigrator {
    fn sources(&self, target: &str) -> Vec<SourceOption> {
        candidates(&DiscoveryInput::from_env(), Path::exists)
            .into_iter()
            .map(|candidate| {
                let host = candidate.endpoint.to_string();
                let found = match candidate.source {
                    CandidateSource::DockerHost => "DOCKER_HOST",
                    CandidateSource::Context => "Current context",
                    CandidateSource::Socket => "Socket",
                };
                let label = match product(&host) {
                    Some(product) => format!("{product} · {found}"),
                    None => found.to_string(),
                };
                SourceOption { label, host }
            })
            .filter(|option| option.host != target)
            .collect()
    }

    fn open(&self, source: &str, target: &str) -> Result<Arc<dyn MigrationSession>, EngineError> {
        let parse = |host: &str| {
            Endpoint::parse(host).map_err(|error| EngineError::Unreachable(error.to_string()))
        };
        let session = DockerSession::connect(&parse(source)?, &parse(target)?)?;
        if session.same_engine() {
            return Err(EngineError::Api(
                "The source and the target are the same engine. Pick another source.".into(),
            ));
        }
        Ok(Arc::new(session))
    }
}

/// The product behind a well-known socket path.
pub fn product(host: &str) -> Option<&'static str> {
    [
        ("/.rd/", "Rancher Desktop"),
        ("/.docker/run/", "Docker Desktop"),
        ("/.orbstack/", "OrbStack"),
        ("/.colima/", "Colima"),
        ("/Captain/lima/", "Captain Engine"),
    ]
    .into_iter()
    .find(|(path, _)| host.contains(path))
    .map(|(_, product)| product)
}

#[cfg(test)]
mod tests;
