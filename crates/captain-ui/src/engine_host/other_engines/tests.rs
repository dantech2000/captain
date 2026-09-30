// The test links sockets, which needs a Unix file system.
#![cfg(unix)]

use super::other_engines;
use crate::settings::DetectedEndpoint;

fn endpoint(host: &str) -> DetectedEndpoint {
    DetectedEndpoint {
        source: "Socket".into(),
        host: host.to_string().into(),
    }
}

#[test]
fn leaves_out_captain_engine_and_links_to_its_socket() {
    let dir = std::env::temp_dir().join(format!("captain-other-engines-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let socket = dir.join("docker.sock");
    std::fs::write(&socket, "").unwrap();
    let link = dir.join("link.sock");
    std::fs::remove_file(&link).ok();
    std::os::unix::fs::symlink(&socket, &link).unwrap();
    let captain = format!("unix://{}", socket.display());
    let detected = vec![
        endpoint(&captain),
        endpoint(&format!("unix://{}", link.display())),
        endpoint("unix:///nowhere/other.sock"),
    ];

    let others = other_engines(detected, Some(&captain));
    std::fs::remove_dir_all(&dir).ok();

    let hosts: Vec<_> = others.iter().map(|e| e.host.to_string()).collect();
    assert_eq!(hosts, ["unix:///nowhere/other.sock"]);
}
