use super::*;

#[test]
fn a_registry_helper_wins_over_the_store_and_the_file() {
    let config = DockerConfig::parse(
        r#"{"auths": {"gcr.io": {}}, "credsStore": "osxkeychain",
            "credHelpers": {"gcr.io": "gcloud"}}"#,
    )
    .unwrap();
    let helper = |name: &str, server: &str| CredentialSource::Helper {
        name: name.into(),
        server_address: server.into(),
    };
    assert_eq!(config.source("gcr.io"), helper("gcloud", "gcr.io"));
    assert_eq!(
        config.source("docker.io"),
        helper("osxkeychain", "https://index.docker.io/v1/")
    );
}

#[test]
fn without_helpers_the_file_entry_is_the_login() {
    // "ZGFuOnB3" is base64 of "dan:pw".
    let config =
        DockerConfig::parse(r#"{"auths": {"https://index.docker.io/v1/": {"auth": "ZGFuOnB3"}}}"#)
            .unwrap();
    let CredentialSource::Stored(login) = config.source("docker.io") else {
        panic!("expected a stored login");
    };
    assert_eq!(login.username.as_deref(), Some("dan"));
    assert_eq!(login.password.as_deref(), Some("pw"));
    assert_eq!(login.server_address, "https://index.docker.io/v1/");
    assert_eq!(config.source("ghcr.io"), CredentialSource::None);
}
