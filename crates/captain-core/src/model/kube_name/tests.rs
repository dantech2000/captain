use super::kube_display_name;

#[test]
fn reads_pod_and_container_from_the_cri_dockerd_name() {
    let app = "k8s_web_web-7d4b9c-x2x9k_default_0f1c2d3e-aaaa-bbbb_0";
    assert_eq!(
        kube_display_name(app).as_deref(),
        Some("web-7d4b9c-x2x9k/web")
    );
    let sandbox = "k8s_POD_web-7d4b9c-x2x9k_default_0f1c2d3e-aaaa-bbbb_0";
    assert_eq!(
        kube_display_name(sandbox).as_deref(),
        Some("web-7d4b9c-x2x9k (sandbox)")
    );
    assert_eq!(kube_display_name("shop-web-1"), None);
}
