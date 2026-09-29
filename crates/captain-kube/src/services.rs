//! Lists Services as `captain-core` types.

use captain_core::HostError;
use captain_core::kubernetes::{KubeService, ServicePort};
use k8s_openapi::api::core::v1::Service;
use kube::api::ListParams;
use kube::{Api, Client};

use crate::client::error;

/// Every Service with a TCP port, by namespace and name.
pub async fn list(client: Client) -> Result<Vec<KubeService>, HostError> {
    let api: Api<Service> = Api::all(client);
    let list = api.list(&ListParams::default()).await.map_err(error)?;
    let mut services: Vec<KubeService> = list.items.iter().filter_map(convert).collect();
    services.sort_by(|a, b| (&a.namespace, &a.name).cmp(&(&b.namespace, &b.name)));
    Ok(services)
}

fn convert(service: &Service) -> Option<KubeService> {
    let ports: Vec<ServicePort> = service
        .spec
        .as_ref()?
        .ports
        .iter()
        .flatten()
        .filter(|port| port.protocol.as_deref().unwrap_or("TCP") == "TCP")
        .filter_map(|port| {
            Some(ServicePort {
                name: port.name.clone(),
                port: u16::try_from(port.port).ok()?,
                protocol: "TCP".into(),
            })
        })
        .collect();
    (!ports.is_empty()).then(|| KubeService {
        namespace: service.metadata.namespace.clone().unwrap_or_default(),
        name: service.metadata.name.clone().unwrap_or_default(),
        ports,
    })
}
