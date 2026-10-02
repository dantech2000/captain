use std::collections::HashMap;

use bollard::models::ContainerSummary;

use super::container;

#[test]
fn reads_the_compose_labels_and_the_kubernetes_namespace() {
    let labels = [
        ("com.docker.compose.project", "shop"),
        ("com.docker.compose.service", "web"),
        ("com.docker.compose.project.working_dir", "/code/shop"),
        (
            "com.docker.compose.project.config_files",
            "/code/shop/compose.yaml,/code/shop/compose.override.yaml",
        ),
    ];
    let summary = ContainerSummary {
        names: Some(vec!["/shop-web-1".into()]),
        labels: Some(HashMap::from(labels.map(|(k, v)| (k.into(), v.into())))),
        ..Default::default()
    };

    let c = container(summary);

    assert_eq!(c.name, "shop-web-1");
    assert_eq!(c.compose_project.as_deref(), Some("shop"));
    assert_eq!(c.compose.service.as_deref(), Some("web"));
    assert_eq!(c.compose.working_dir.as_deref(), Some("/code/shop"));
    assert_eq!(
        c.compose.config_files,
        [
            "/code/shop/compose.yaml",
            "/code/shop/compose.override.yaml"
        ]
    );
    // A plain container has no compose labels.
    let c = container(ContainerSummary::default());
    assert_eq!(c.compose_project, None);
    assert_eq!(c.compose, Default::default());
    let summary = ContainerSummary {
        labels: Some(HashMap::from([(
            "io.kubernetes.pod.namespace".to_string(),
            "kube-system".to_string(),
        )])),
        ..Default::default()
    };
    let c = container(summary);
    assert_eq!(c.kube_namespace.as_deref(), Some("kube-system"));
    assert!(c.is_kubernetes());
}
