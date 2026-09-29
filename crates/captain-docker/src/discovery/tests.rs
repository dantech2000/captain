use std::path::{Path, PathBuf};

use super::{DiscoveryError, DiscoveryInput, discover};
use crate::Endpoint;

fn input(docker_host: Option<&str>) -> DiscoveryInput {
    DiscoveryInput {
        docker_host: docker_host.map(Into::into),
        docker_context: None,
        home: Some(PathBuf::from("/nonexistent-home")),
    }
}

#[test]
fn docker_host_wins() {
    let endpoint = discover(&input(Some("tcp://1.2.3.4:2375")), |_| true);
    assert_eq!(endpoint, Ok(Endpoint::Tcp("tcp://1.2.3.4:2375".into())));
}

#[test]
fn unsupported_docker_host_is_an_error() {
    let endpoint = discover(&input(Some("ssh://me@host")), |_| true);
    assert!(matches!(endpoint, Err(DiscoveryError::Unsupported(_))));
}

#[cfg(unix)]
#[test]
fn first_existing_home_socket_wins() {
    let orbstack = Path::new("/nonexistent-home/.orbstack/run/docker.sock");
    let endpoint = discover(&input(None), |path| path == orbstack);
    assert_eq!(endpoint, Ok(Endpoint::Unix(orbstack.into())));
}

#[cfg(unix)]
#[test]
fn falls_back_to_system_socket() {
    let system = Path::new("/var/run/docker.sock");
    let endpoint = discover(&input(None), |path| path == system);
    assert_eq!(endpoint, Ok(Endpoint::Unix(system.into())));
}

#[cfg(unix)]
#[test]
fn nothing_found_is_an_error() {
    assert_eq!(
        discover(&input(None), |_| false),
        Err(DiscoveryError::NotFound)
    );
}
