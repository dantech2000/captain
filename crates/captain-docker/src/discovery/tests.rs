#[cfg(unix)]
use std::path::Path;
use std::path::PathBuf;

use super::{CandidateSource, DiscoveryError, DiscoveryInput, candidates, discover};
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

#[cfg(windows)]
#[test]
fn windows_falls_back_to_named_pipe() {
    let endpoint = discover(&input(None), |_| false);
    assert_eq!(
        endpoint,
        Ok(Endpoint::NamedPipe("//./pipe/docker_engine".into()))
    );
}

#[cfg(unix)]
#[test]
fn candidates_list_host_then_existing_sockets() {
    use super::Candidate;

    let colima = Path::new("/nonexistent-home/.colima/default/docker.sock");
    let system = Path::new("/var/run/docker.sock");
    let found = candidates(&input(Some("tcp://1.2.3.4:2375")), |path| {
        path == colima || path == system
    });
    assert_eq!(
        found,
        [
            Candidate {
                source: CandidateSource::DockerHost,
                endpoint: Endpoint::Tcp("tcp://1.2.3.4:2375".into()),
            },
            Candidate {
                source: CandidateSource::Socket,
                endpoint: Endpoint::Unix(colima.into()),
            },
            Candidate {
                source: CandidateSource::Socket,
                endpoint: Endpoint::Unix(system.into()),
            },
        ]
    );
}

#[test]
fn candidates_skip_unsupported_hosts() {
    let found = candidates(&input(Some("ssh://me@host")), |_| false);
    assert!(
        found
            .iter()
            .all(|candidate| candidate.source != CandidateSource::DockerHost)
    );
}

#[cfg(unix)]
#[test]
fn candidates_drop_duplicates() {
    let found = candidates(&input(Some("unix:///var/run/docker.sock")), |_| false);
    assert_eq!(found.len(), 1);
    let found = candidates(&input(Some("unix:///var/run/docker.sock")), |path| {
        path == Path::new("/var/run/docker.sock")
    });
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].source, CandidateSource::DockerHost);
}

#[cfg(unix)]
#[test]
fn candidates_empty_when_nothing_exists() {
    assert!(candidates(&input(None), |_| false).is_empty());
}
