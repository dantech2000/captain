use super::*;

#[test]
fn a_refused_push_suggests_docker_login() {
    assert_eq!(
        push_error(
            "denied: requested access to the resource is denied",
            "docker.io",
            false
        ),
        "denied: requested access to the resource is denied. Captain found no login for \
         docker.io. Run `docker login docker.io` in a terminal, then push again."
    );
    assert_eq!(
        push_error("unauthorized: authentication required", "ghcr.io", true),
        "unauthorized: authentication required. Run `docker login ghcr.io` in a terminal, \
         then push again."
    );
    assert_eq!(
        push_error("connection refused", "ghcr.io", false),
        "connection refused"
    );
}
