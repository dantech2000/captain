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
fn a_link_to_captain_is_captain() {
    let link = classify(SocketProbe::Link(CAPTAIN.into()));
    assert_eq!(link, SocketLink::Captain);
    assert_eq!(link.action(), Some(LinkAction::Unlink));
    assert!(!link.needs_confirmation());
}

#[test]
fn a_missing_socket_links_without_asking() {
    let link = classify(SocketProbe::Missing);
    assert_eq!(link.action(), Some(LinkAction::Link));
    assert!(!link.needs_confirmation());
}

#[test]
fn another_engine_needs_confirmation() {
    let other = classify(SocketProbe::Link("/Users/me/.rd/docker.sock".into()));
    assert_eq!(
        other,
        SocketLink::OtherLink("/Users/me/.rd/docker.sock".into())
    );
    assert!(other.needs_confirmation());
    let socket = classify(SocketProbe::Socket);
    assert_eq!(socket.action(), Some(LinkAction::Link));
    assert!(socket.needs_confirmation());
}

#[test]
fn a_relative_link_is_read_from_var_run() {
    let link = classify(SocketProbe::Link("docker.sock.real".into()));
    assert_eq!(
        link,
        SocketLink::OtherLink(Path::new("/var/run").join("docker.sock.real"))
    );
}

#[test]
fn a_plain_file_is_left_alone() {
    assert_eq!(classify(SocketProbe::Other).action(), None);
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
