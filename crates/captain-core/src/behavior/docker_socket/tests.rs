use std::path::{Path, PathBuf};

use super::{
    Elevation, LinkAction, SocketLink, SocketProbe, failure_message, link_command, socket_path,
    unlink_command,
};

const CAPTAIN: &str = "/Users/me/.captain/lima/captain/sock/docker.sock";

fn classify(probe: SocketProbe) -> SocketLink {
    SocketLink::classify(&probe, Path::new(CAPTAIN))
}

#[test]
fn each_probe_gets_its_action_and_only_another_engine_asks_first() {
    let rd = PathBuf::from("/Users/me/.rd/docker.sock");
    let cases = [
        (
            SocketProbe::Link(CAPTAIN.into()),
            SocketLink::Captain,
            Some(LinkAction::Unlink),
            false,
        ),
        (
            SocketProbe::Missing,
            SocketLink::Missing,
            Some(LinkAction::Link),
            false,
        ),
        (
            SocketProbe::Link(rd.clone()),
            SocketLink::OtherLink(rd),
            Some(LinkAction::Link),
            true,
        ),
        // A relative link is read from /var/run.
        (
            SocketProbe::Link("docker.sock.real".into()),
            SocketLink::OtherLink(Path::new("/var/run").join("docker.sock.real")),
            Some(LinkAction::Link),
            true,
        ),
        (
            SocketProbe::Socket,
            SocketLink::Socket,
            Some(LinkAction::Link),
            true,
        ),
        // A plain file is left alone.
        (SocketProbe::Other, SocketLink::Other, None, false),
    ];
    for (probe, link, action, asks) in cases {
        let found = classify(probe);
        assert_eq!(found, link);
        assert_eq!(found.action(), action, "{link:?}");
        assert_eq!(found.needs_confirmation(), asks, "{link:?}");
    }
}

#[test]
fn only_unix_endpoints_have_a_socket() {
    assert_eq!(
        socket_path(&format!("unix://{CAPTAIN}")),
        Some(PathBuf::from(CAPTAIN))
    );
    assert_eq!(socket_path("tcp://10.0.0.5:2375"), None);
}

#[test]
fn apple_script_passes_the_paths_as_arguments() {
    let target = "/Users/o'neil/sock/docker.sock";
    let command = link_command(
        Elevation::AppleScript,
        Path::new(target),
        &SocketLink::Missing,
    );
    assert_eq!(command.program, "/usr/bin/osascript");
    let script = &command.args[..6];
    assert!(
        script
            .iter()
            .any(|arg| arg.contains("quoted form of item 7 of argv"))
    );
    assert!(!script.iter().any(|arg| arg.contains("o'neil")));
    assert_eq!(
        &command.args[8..],
        ["link", "/var/run/docker.sock", target, "missing", ""]
    );
}

#[test]
fn pkexec_runs_the_script_with_sh() {
    let command = unlink_command(Elevation::Pkexec, Path::new(CAPTAIN));
    assert_eq!(command.program, "pkexec");
    assert_eq!(&command.args[..2], ["/bin/sh", "-c"]);
    assert_eq!(
        &command.args[3..],
        ["sh", "unlink", "/var/run/docker.sock", CAPTAIN]
    );
}

#[cfg(unix)]
#[test]
fn the_script_leaves_a_socket_that_changed_after_the_check() {
    use super::script_argv;

    let dir = std::env::temp_dir().join(format!("captain-sockscript-{}", std::process::id()));
    std::fs::remove_dir_all(&dir).ok();
    std::fs::create_dir_all(&dir).unwrap();
    let socket = dir.join("docker.sock");
    let (captain, other) = (dir.join("captain.sock"), dir.join("other.sock"));
    let run = |replacing: Option<&SocketLink>| {
        let argv = script_argv(&socket, &captain, replacing);
        std::process::Command::new("/bin/sh")
            .arg("-c")
            .args(&argv)
            .status()
            .unwrap()
            .success()
    };
    // Another engine linked it while the prompt was open: unlink leaves it.
    std::os::unix::fs::symlink("other.sock", &socket).unwrap();
    assert!(!run(None));
    assert_eq!(
        std::fs::read_link(&socket).unwrap(),
        Path::new("other.sock")
    );
    // Linking over a missing socket refuses, and linking over the confirmed one works.
    assert!(!run(Some(&SocketLink::Missing)));
    assert!(run(Some(&SocketLink::OtherLink(other.clone()))));
    assert_eq!(std::fs::read_link(&socket).unwrap(), captain);
    assert!(run(None));
    assert!(std::fs::symlink_metadata(&socket).is_err());
    std::fs::write(&socket, "a plain file").unwrap();
    assert!(!run(Some(&SocketLink::Missing)));
    assert_eq!(std::fs::read_to_string(&socket).unwrap(), "a plain file");
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn cancel_has_a_plain_message() {
    let stderr = "0:120: execution error: User canceled. (-128)";
    assert_eq!(failure_message(stderr), "You canceled the request.");
    assert_eq!(failure_message("  "), "The command failed.");
}
