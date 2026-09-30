use super::inspect_report;
use crate::agent_tools::test_fleet::fleet;
use crate::model::{ContainerDetail, EnvVar};

#[test]
fn masks_secret_environment_values() {
    let container = &fleet(1)[0];
    let detail = ContainerDetail {
        command: "nginx -g 'daemon off;'".into(),
        env: [
            "POSTGRES_PASSWORD=hunter2",
            "DATABASE_URL=postgres://app:s3cret@db/shop",
            "PORT=80",
        ]
        .map(EnvVar::parse)
        .to_vec(),
        ..ContainerDetail::default()
    };
    let report = inspect_report(container, &detail);
    let values: Vec<&str> = report.env.iter().map(|e| e.value.as_str()).collect();
    assert_eq!(
        values,
        ["[masked]", "postgres://app:[masked]@db/shop", "80"]
    );
    assert_eq!(report.restart_policy, "no");
    let text = report.text();
    assert!(!text.contains("hunter2") && !text.contains("s3cret"));
    assert!(text.contains("UNTRUSTED CONTAINER OUTPUT"));
}
