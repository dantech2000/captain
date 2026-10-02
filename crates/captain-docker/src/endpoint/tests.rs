use std::path::PathBuf;

use super::Endpoint;

#[test]
fn parses_and_prints_each_supported_scheme_and_rejects_ssh() {
    assert_eq!(
        Endpoint::parse("unix:///var/run/docker.sock"),
        Ok(Endpoint::Unix(PathBuf::from("/var/run/docker.sock")))
    );
    assert_eq!(
        Endpoint::parse("npipe:////./pipe/docker_engine"),
        Ok(Endpoint::NamedPipe("//./pipe/docker_engine".into()))
    );
    assert_eq!(
        Endpoint::parse("tcp://10.0.0.5:2375"),
        Ok(Endpoint::Tcp("tcp://10.0.0.5:2375".into()))
    );
    assert!(Endpoint::parse("ssh://me@host").is_err());
    for host in [
        "unix:///var/run/docker.sock",
        "npipe:////./pipe/docker_engine",
    ] {
        assert_eq!(Endpoint::parse(host).unwrap().to_string(), host);
    }
}
