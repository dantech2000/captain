//! Converts a [`RunSpec`] into the body of a container `create` request.

use std::collections::HashMap;

use bollard::models::{
    ContainerCreateBody, HostConfig, PortBinding, RestartPolicy as EngineRestart,
    RestartPolicyNameEnum,
};
use captain_core::model::{RestartPolicy, RunSpec};

/// The create body for `spec`: the image, env, exposed and published ports, the
/// restart policy, and auto-remove.
pub fn run_body(spec: &RunSpec) -> ContainerCreateBody {
    let mut port_bindings = HashMap::new();
    for port in &spec.ports {
        port_bindings
            .entry(port.container_key())
            .or_insert_with(|| Some(Vec::new()))
            .get_or_insert_with(Vec::new)
            .push(PortBinding {
                host_ip: None,
                host_port: Some(port.host.to_string()),
            });
    }
    let mut exposed_ports: Vec<String> = port_bindings.keys().cloned().collect();
    exposed_ports.sort();

    ContainerCreateBody {
        image: Some(spec.image.clone()),
        env: (!spec.env.is_empty()).then(|| {
            spec.env
                .iter()
                .map(|var| format!("{}={}", var.key, var.value))
                .collect()
        }),
        exposed_ports: (!exposed_ports.is_empty()).then_some(exposed_ports),
        host_config: Some(HostConfig {
            port_bindings: (!port_bindings.is_empty()).then_some(port_bindings),
            auto_remove: Some(spec.auto_remove),
            restart_policy: Some(EngineRestart {
                name: Some(restart_name(spec.restart)),
                maximum_retry_count: None,
            }),
            ..HostConfig::default()
        }),
        ..ContainerCreateBody::default()
    }
}

/// The engine's name for `policy`.
pub fn restart_name(policy: RestartPolicy) -> RestartPolicyNameEnum {
    match policy {
        RestartPolicy::No => RestartPolicyNameEnum::NO,
        RestartPolicy::UnlessStopped => RestartPolicyNameEnum::UNLESS_STOPPED,
        RestartPolicy::Always => RestartPolicyNameEnum::ALWAYS,
        RestartPolicy::OnFailure => RestartPolicyNameEnum::ON_FAILURE,
    }
}

#[cfg(test)]
mod tests;
