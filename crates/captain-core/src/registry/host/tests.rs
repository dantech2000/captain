use super::*;

#[test]
fn the_registry_is_the_first_part_only_when_it_looks_like_a_host() {
    assert_eq!(registry_host("nginx:alpine"), "docker.io");
    assert_eq!(registry_host("team/app:1"), "docker.io");
    assert_eq!(registry_host("ghcr.io/team/app:1"), "ghcr.io");
    assert_eq!(registry_host("localhost:5000/app"), "localhost:5000");
    assert_eq!(registry_host("localhost/app"), "localhost");
}

#[test]
fn config_keys_become_host_names() {
    assert_eq!(normalize_host("https://index.docker.io/v1/"), "docker.io");
    assert_eq!(
        normalize_host("http://registry.example.com/v2"),
        "registry.example.com"
    );
    assert_eq!(normalize_host("gcr.io"), "gcr.io");
    assert_eq!(server_address("docker.io"), "https://index.docker.io/v1/");
}
