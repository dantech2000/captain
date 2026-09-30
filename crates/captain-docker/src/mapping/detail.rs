use bollard::models::{ContainerInspectResponse, HealthcheckResult};
use captain_core::model::{ContainerDetail, EnvVar, HealthCheck, Mount};

pub fn detail(response: ContainerInspectResponse) -> ContainerDetail {
    let config = response.config.unwrap_or_default();
    let command = config
        .entrypoint
        .unwrap_or_default()
        .into_iter()
        .chain(config.cmd.unwrap_or_default())
        .collect::<Vec<_>>()
        .join(" ");
    let mounts = response
        .mounts
        .unwrap_or_default()
        .into_iter()
        .map(|m| Mount {
            source: m.name.or(m.source).unwrap_or_default(),
            destination: m.destination.unwrap_or_default(),
        })
        .collect();
    let mut networks: Vec<String> = response
        .network_settings
        .and_then(|n| n.networks)
        .map(|n| n.into_keys().collect())
        .unwrap_or_default();
    networks.sort();
    let host_config = response.host_config.unwrap_or_default();
    let memory_limit = host_config.memory.unwrap_or_default();
    let restart_policy = host_config
        .restart_policy
        .and_then(|p| p.name)
        .map(|name| name.to_string())
        .unwrap_or_default();
    let state = response.state.unwrap_or_default();
    let health_checks = state
        .health
        .and_then(|h| h.log)
        .unwrap_or_default()
        .iter()
        .map(check)
        .collect();

    ContainerDetail {
        id: response.id.unwrap_or_default(),
        command,
        env: config
            .env
            .unwrap_or_default()
            .iter()
            .map(|e| EnvVar::parse(e))
            .collect(),
        mounts,
        networks,
        restart_policy,
        health_checks,
        started_at: state.started_at.unwrap_or_default(),
        oom_killed: state.oom_killed.unwrap_or_default(),
        memory_limit,
        restart_count: response.restart_count.unwrap_or_default(),
    }
}

fn check(result: &HealthcheckResult) -> HealthCheck {
    HealthCheck {
        passed: result.exit_code == Some(0),
    }
}
