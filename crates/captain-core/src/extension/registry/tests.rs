use super::{RegistryRepository, next_page, token_url};

#[test]
fn a_repository_finds_its_registry_token_and_next_page() {
    let hub = RegistryRepository::parse("docker/disk-usage-extension").unwrap();
    assert_eq!(
        hub.tags_url(),
        "https://registry-1.docker.io/v2/docker/disk-usage-extension/tags/list?n=1000"
    );
    assert_eq!(
        RegistryRepository::parse("nginx").unwrap().path,
        "library/nginx"
    );
    let ghcr = RegistryRepository::parse("ghcr.io/acme/ext").unwrap();
    assert_eq!(
        (ghcr.host.as_str(), ghcr.path.as_str()),
        ("ghcr.io", "acme/ext")
    );

    let challenge = r#"Bearer realm="https://auth.docker.io/token",service="registry.docker.io",scope="repository:docker/x:pull,push""#;
    assert_eq!(
        token_url(challenge).as_deref(),
        Some(
            "https://auth.docker.io/token?service=registry.docker.io&scope=repository:docker/x:pull%2Cpush"
        )
    );
    assert_eq!(token_url(r#"Basic realm="x""#), None);

    let link = r#"</v2/p/ext/tags/list?last=0.2.0&n=5>; rel="next""#;
    assert_eq!(
        next_page(link, "registry-1.docker.io").as_deref(),
        Some("https://registry-1.docker.io/v2/p/ext/tags/list?last=0.2.0&n=5")
    );
}
