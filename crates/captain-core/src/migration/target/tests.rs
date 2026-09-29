use super::is_captain_engine;

#[test]
fn knows_the_captain_socket() {
    assert!(is_captain_engine(
        "unix:///Users/me/Library/Application Support/Captain/lima/captain/sock/docker.sock"
    ));
    assert!(!is_captain_engine("unix:///Users/me/.rd/docker.sock"));
    assert!(!is_captain_engine("tcp://10.0.0.5:2375"));
}
