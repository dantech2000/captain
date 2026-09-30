use bollard::models::RestartPolicyNameEnum;
use captain_core::model::{ResourceUpdate, RestartPolicy};

use super::update_body;

#[test]
fn sets_only_the_given_fields_and_doubles_swap() {
    let body = update_body(ResourceUpdate {
        memory: Some(512 << 20),
        nano_cpus: None,
        restart_policy: Some(RestartPolicy::UnlessStopped),
    });
    assert_eq!(body.memory, Some(512 << 20));
    assert_eq!(body.memory_swap, Some(1024 << 20));
    assert_eq!(body.nano_cpus, None);
    assert_eq!(
        body.restart_policy.and_then(|p| p.name),
        Some(RestartPolicyNameEnum::UNLESS_STOPPED)
    );
    let cpus = update_body(ResourceUpdate {
        nano_cpus: Some(1_500_000_000),
        ..ResourceUpdate::default()
    });
    assert_eq!(cpus.nano_cpus, Some(1_500_000_000));
    assert_eq!((cpus.memory, cpus.memory_swap), (None, None));
    assert!(cpus.restart_policy.is_none());
}
