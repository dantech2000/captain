use std::path::Path;

use captain_core::{EngineHost, GIB, HostResources, HostStatus};
use futures::StreamExt;
use futures::executor::block_on;

use super::{SystemHost, socket_status};

fn host() -> SystemHost {
    SystemHost::new(HostResources::recommended(8, 16 * GIB))
}

#[test]
fn status_follows_the_socket() {
    assert_eq!(
        socket_status(Path::new("/definitely/not/here.sock")),
        HostStatus::Stopped
    );
    assert_eq!(socket_status(Path::new("/")), HostStatus::Running);
}

#[test]
fn reports_the_system_socket_and_refuses_control() {
    let host = host();
    assert!(!host.can_control());
    assert_eq!(
        host.endpoint().as_deref(),
        Some("unix:///var/run/docker.sock")
    );
    assert!(block_on(host.stop()).is_err());
    let first = block_on(host.start().next()).unwrap();
    assert!(first.is_err());
}
