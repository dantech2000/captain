use std::path::{Path, PathBuf};

use super::{
    Elevation, LinkAction, SocketLink, SocketProbe, failure_message, link_command, socket_path,
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
fn apple_script_passes_the_path_as_an_argument() {
    let target = "/Users/o'neil/sock/docker.sock";
    let command = link_command(Elevation::AppleScript, Path::new(target));
    assert_eq!(command.program, "/usr/bin/osascript");
    assert_eq!(command.args.last().map(String::as_str), Some(target));
    assert!(
        command
            .args
            .iter()
            .any(|arg| arg.contains("quoted form of item 1 of argv"))
    );
    assert!(
        !command.args[..command.args.len() - 1]
            .iter()
            .any(|arg| arg.contains("o'neil"))
    );
}

#[test]
fn pkexec_runs_ln_directly() {
    let command = link_command(Elevation::Pkexec, Path::new(CAPTAIN));
    assert_eq!(command.program, "pkexec");
    assert_eq!(
        command.args,
        ["/bin/ln", "-sfn", CAPTAIN, "/var/run/docker.sock"]
    );
}

#[test]
fn cancel_has_a_plain_message() {
    let stderr = "0:120: execution error: User canceled. (-128)";
    assert_eq!(failure_message(stderr), "You canceled the request.");
    assert_eq!(failure_message("  "), "The command failed.");
}
