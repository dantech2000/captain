//! Finds the pod and port behind a Service port, as `kubectl port-forward svc/...`
//! does: a running pod that the selector matches, and the `targetPort`.

use captain_core::HostError;
use captain_core::kubernetes::{ForwardKey, TargetPort, resolve_target};
use k8s_openapi::api::core::v1::{Pod, Service};
use k8s_openapi::apimachinery::pkg::util::intstr::IntOrString;
use kube::api::ListParams;
use kube::{Api, Client};

use crate::client::error;

/// A pod name and the pod port to connect to.
pub struct PodTarget {
    pub pod: String,
    pub port: u16,
}

pub async fn find(client: &Client, key: &ForwardKey) -> Result<PodTarget, HostError> {
    let services: Api<Service> = Api::namespaced(client.clone(), &key.namespace);
    let service = services.get(&key.service).await.map_err(error)?;
    let spec = service.spec.unwrap_or_default();
    let service_port = spec
        .ports
        .iter()
        .flatten()
        .find(|port| i32::from(key.port) == port.port)
        .ok_or_else(|| HostError(format!("{} has no port {}.", key.service, key.port)))?;
    let target = match &service_port.target_port {
        Some(IntOrString::Int(port)) => {
            TargetPort::Number(u16::try_from(*port).unwrap_or(key.port))
        }
        Some(IntOrString::String(name)) => TargetPort::Name(name.clone()),
        None => TargetPort::Same,
    };
    let selector = spec
        .selector
        .filter(|selector| !selector.is_empty())
        .ok_or_else(|| HostError(format!("{} has no pod selector.", key.service)))?
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join(",");
    let pods: Api<Pod> = Api::namespaced(client.clone(), &key.namespace);
    let list = pods
        .list(&ListParams::default().labels(&selector))
        .await
        .map_err(error)?;
    let pod = list
        .items
        .into_iter()
        .find(is_running)
        .ok_or_else(|| HostError(format!("No pod of {} runs.", key.service)))?;
    let port = resolve_target(key.port, &target, &container_ports(&pod))
        .ok_or_else(|| HostError(format!("The pod has no port for {}.", key.port)))?;
    Ok(PodTarget {
        pod: pod.metadata.name.unwrap_or_default(),
        port,
    })
}

fn is_running(pod: &Pod) -> bool {
    pod.metadata.deletion_timestamp.is_none()
        && pod
            .status
            .as_ref()
            .and_then(|status| status.phase.as_deref())
            == Some("Running")
}

fn container_ports(pod: &Pod) -> Vec<(Option<String>, u16)> {
    pod.spec
        .iter()
        .flat_map(|spec| &spec.containers)
        .flat_map(|container| container.ports.iter().flatten())
        .filter_map(|port| Some((port.name.clone(), u16::try_from(port.container_port).ok()?)))
        .collect()
}
