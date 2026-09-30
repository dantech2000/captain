/// The name to show for a Kubernetes pod container: `pod/container`, read from the
/// name cri-dockerd gives it, `k8s_<container>_<pod>_<namespace>_<uid>_<attempt>`.
/// The pod's sandbox (`k8s_POD_…`) gives `pod (sandbox)`. `None` for any other name.
pub fn kube_display_name(name: &str) -> Option<String> {
    let rest = name.strip_prefix("k8s_")?;
    let parts: Vec<&str> = rest.split('_').collect();
    // Pod and container names cannot hold `_`, so the parts are fixed.
    let [container, pod, _namespace, _uid, _attempt] = parts.as_slice() else {
        return None;
    };
    if container.is_empty() || pod.is_empty() {
        return None;
    }
    Some(if *container == "POD" {
        format!("{pod} (sandbox)")
    } else {
        format!("{pod}/{container}")
    })
}

#[cfg(test)]
mod tests;
