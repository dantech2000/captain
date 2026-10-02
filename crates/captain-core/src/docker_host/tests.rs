use super::cli_host;

#[test]
fn turns_http_into_tcp() {
    assert_eq!(cli_host("http://10.0.0.5:2375"), "tcp://10.0.0.5:2375");
    assert_eq!(
        cli_host("unix:///var/run/docker.sock"),
        "unix:///var/run/docker.sock"
    );
}
