use std::path::Path;

use super::SshTarget;

#[test]
fn parses_user_host_port_and_socket() {
    let target = SshTarget::parse("ssh://me@box.lan:2222/run/user/1000/docker.sock").unwrap();
    assert_eq!(target.user.as_deref(), Some("me"));
    assert_eq!(target.host, "box.lan");
    assert_eq!(target.port, Some(2222));
    assert_eq!(target.remote_socket(), "/run/user/1000/docker.sock");
}

#[test]
fn host_alone_uses_default_socket() {
    let target = SshTarget::parse("ssh://box/").unwrap();
    assert_eq!((target.user.as_deref(), target.port), (None, None));
    assert_eq!(target.remote_socket(), "/var/run/docker.sock");
}

#[test]
fn parses_ipv6_and_round_trips() {
    let url = "ssh://me@[fe80::1]:22";
    let target = SshTarget::parse(url).unwrap();
    assert_eq!(target.host, "fe80::1");
    assert_eq!(target.to_string(), url);
}

#[test]
fn rejects_bad_urls() {
    for url in [
        "tcp://box",
        "ssh://",
        "ssh://me:secret@box",
        "ssh://box?x=1",
        "ssh://box#frag",
        "ssh://box:99999",
        "ssh://-oProxyCommand=evil",
        "ssh://@box",
    ] {
        assert!(SshTarget::parse(url).is_err(), "{url} should not parse");
    }
}

#[test]
fn tunnel_args_forward_local_socket_in_batch_mode() {
    let target = SshTarget::parse("ssh://me@box:2222").unwrap();
    let args = target.tunnel_args(Path::new("/tmp/c/docker.sock"));
    assert_eq!(args[0], "-nNT");
    assert!(args.contains(&"BatchMode=yes".to_string()));
    assert!(args.contains(&"ExitOnForwardFailure=yes".to_string()));
    assert!(args.ends_with(&[
        "/tmp/c/docker.sock:/var/run/docker.sock".into(),
        "-l".into(),
        "me".into(),
        "-p".into(),
        "2222".into(),
        "--".into(),
        "box".into(),
    ]));
}

#[test]
fn explains_password_prompt() {
    let target = SshTarget::parse("ssh://box").unwrap();
    let message = target.explain_failure("me@box: Permission denied (publickey,password).\n");
    assert!(message.contains("without a password"), "{message}");
    let message = target.explain_failure("ssh: Could not resolve hostname box\n");
    assert_eq!(
        message,
        "SSH to box failed: ssh: Could not resolve hostname box"
    );
}
