//! Kubernetes Services and the port forwards the Port Forwarding page makes.

use crate::HostFuture;

/// A Service and its ports.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KubeService {
    pub namespace: String,
    pub name: String,
    pub ports: Vec<ServicePort>,
}

/// One port of a Service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServicePort {
    pub name: Option<String>,
    pub port: u16,
    pub protocol: String,
}

/// A Service port that can be forwarded.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ForwardKey {
    pub namespace: String,
    pub service: String,
    pub port: u16,
}

/// A forward that runs: connections to `127.0.0.1:<local_port>` reach the Service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Forward {
    pub key: ForwardKey,
    pub local_port: u16,
}

/// A Service port's `targetPort`: a number, a container port name, or unset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TargetPort {
    Number(u16),
    Name(String),
    /// Unset: the target is the Service port itself.
    Same,
}

/// The pod port behind `service_port`, given the pod's named container ports.
pub fn resolve_target(
    service_port: u16,
    target: &TargetPort,
    container_ports: &[(Option<String>, u16)],
) -> Option<u16> {
    match target {
        TargetPort::Number(port) => Some(*port),
        TargetPort::Same => Some(service_port),
        TargetPort::Name(name) => container_ports
            .iter()
            .find(|(port_name, _)| port_name.as_deref() == Some(name))
            .map(|(_, port)| *port),
    }
}

/// The local ports a user may pick: above 1024, as in Rancher Desktop, so no
/// administrator rights are needed.
pub fn check_local_port(port: u16) -> Result<(), String> {
    if port > 1024 {
        Ok(())
    } else {
        Err("Pick a port above 1024.".into())
    }
}

/// Lists Services and forwards their ports to `127.0.0.1` while Captain runs.
pub trait PortForwarding: Send + Sync + 'static {
    /// All Services with TCP ports, by namespace and name.
    fn services(&self) -> HostFuture<Vec<KubeService>>;

    /// Starts a forward on `local_port`, or a free port when it is `None`.
    fn forward(&self, key: ForwardKey, local_port: Option<u16>) -> HostFuture<Forward>;

    /// Stops a forward. Open connections end with it.
    fn stop(&self, key: &ForwardKey);

    /// The forwards that run now.
    fn forwards(&self) -> Vec<Forward>;
}

#[cfg(test)]
mod tests;
