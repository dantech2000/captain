use captain_core::{GIB, HostResources};

use super::{disk_size, memory_size, render};

fn resources() -> HostResources {
    HostResources {
        cpus: 4,
        memory_bytes: 8 * GIB,
        disk_bytes: 64 * GIB,
    }
}

#[test]
fn sets_the_machine_mounts_home_and_tmp_and_forwards_the_docker_socket() {
    let yaml = render(&resources(), false);
    for line in [
        "minimumLimaVersion: 2.2.0",
        "- template:_images/ubuntu-lts",
        "vmType: vz",
        "mountType: virtiofs",
        "cpus: 4",
        "memory: 8192MiB",
        "disk: 64GiB",
    ] {
        assert!(
            yaml.lines().any(|l| l == line),
            "missing {line:?} in\n{yaml}"
        );
    }
    assert!(
        yaml.contains("- location: \"~\"\n  writable: true\n"),
        "{yaml}"
    );
    assert!(
        yaml.contains("mountPoint: /tmp/lima\n  writable: true\n"),
        "{yaml}"
    );
    assert!(yaml.contains("guestSocket: \"/var/run/docker.sock\""));
    assert!(yaml.contains("hostSocket: \"{{.Dir}}/sock/docker.sock\""));
    assert!(yaml.contains("host.docker.internal: host.lima.internal"));
    assert!(yaml.contains("curl -fsSL https://get.docker.com | sh"));
    assert!(yaml.contains("containerd-snapshotter"));
}

#[test]
fn rosetta_only_when_asked() {
    assert!(!render(&resources(), false).contains("rosetta"));
    let yaml = render(&resources(), true);
    assert!(yaml.contains("vmOpts:\n  vz:\n    rosetta:\n      enabled: true\n"));
}

#[test]
fn no_tabs_and_every_top_level_key_once() {
    let yaml = render(&resources(), true);
    assert!(!yaml.contains('\t'));
    let keys: Vec<&str> = yaml
        .lines()
        .filter(|l| !l.starts_with([' ', '-', '#']) && l.contains(':'))
        .map(|l| l.split(':').next().unwrap())
        .collect();
    let mut unique = keys.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(keys.len(), unique.len(), "{keys:?}");
}

#[test]
fn sizes_round_down() {
    assert_eq!(memory_size(4 * GIB + GIB / 2), "4608MiB");
    assert_eq!(disk_size(64 * GIB), "64GiB");
    assert_eq!(disk_size(0), "1GiB");
}
