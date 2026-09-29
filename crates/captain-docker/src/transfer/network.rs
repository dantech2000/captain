//! Recreates a user-defined network in the target: same name, driver, options, and
//! labels, and the same subnet when it does not clash with one the target uses.

use bollard::Docker;
use bollard::models::{Ipam, NetworkCreateRequest, NetworkInspect};
use captain_core::EngineError;
use captain_core::migration::TransferEvent;

use super::progress::{self, Events, Outcome};
use super::source::SourceEngine;
use crate::mapping;

/// Creates the network `name` in the target. One that exists there is skipped.
pub async fn copy_network(
    source: &SourceEngine,
    target: &Docker,
    name: &str,
    events: &Events,
) -> Result<Outcome, EngineError> {
    if target.inspect_network(name, None).await.is_ok() {
        return Ok(Outcome::Skipped(
            "A network with this name is already in the target.".into(),
        ));
    }
    let network = source.inspect_network(name).await?;
    let request = create_request(&network, name);
    match target.create_network(request.clone()).await {
        Ok(_) => Ok(Outcome::Copied),
        Err(error) if has_subnet(&request) && is_overlap(&error.to_string()) => {
            let retry = NetworkCreateRequest {
                ipam: None,
                ..request
            };
            target
                .create_network(retry)
                .await
                .map_err(mapping::engine_error)?;
            let note = "Its subnet clashed with the target, so the target picked a new one.";
            progress::send(events, TransferEvent::Note(note.into()));
            Ok(Outcome::Copied)
        }
        Err(error) => Err(mapping::engine_error(error)),
    }
}

/// The create request for a copy of `network`.
pub fn create_request(network: &NetworkInspect, name: &str) -> NetworkCreateRequest {
    let ipam = network.ipam.as_ref().map(|ipam| Ipam {
        driver: ipam.driver.clone(),
        config: ipam.config.clone().filter(|config| !config.is_empty()),
        options: ipam.options.clone(),
    });
    NetworkCreateRequest {
        name: name.into(),
        driver: network.driver.clone(),
        internal: network.internal,
        attachable: network.attachable,
        enable_ipv4: network.enable_ipv4,
        enable_ipv6: network.enable_ipv6,
        ipam,
        options: network.options.clone(),
        labels: network.labels.clone(),
        ..NetworkCreateRequest::default()
    }
}

fn has_subnet(request: &NetworkCreateRequest) -> bool {
    request
        .ipam
        .as_ref()
        .and_then(|ipam| ipam.config.as_ref())
        .is_some_and(|config| !config.is_empty())
}

/// The engine's error for a subnet that another network uses: "Pool overlaps with
/// other one on this address space".
fn is_overlap(message: &str) -> bool {
    message.to_lowercase().contains("overlap")
}

#[cfg(test)]
mod tests;
