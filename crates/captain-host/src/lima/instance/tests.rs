use captain_core::{GIB, HostStatus};

use super::find_instance;

/// Captured from `limactl list --json` (Lima 2.2.0) after `limactl create`, with the
/// large `config` object left out, plus a user instance and a broken one.
const SAMPLE: &str = r#"{"name":"default","hostname":"lima-default","status":"Running","dir":"/Users/ada/.captain/lima/default","vmType":"vz","arch":"aarch64","cpus":4,"memory":4294967296,"disk":107374182400,"protected":false,"limaVersion":"2.2.0"}
{"name":"captain","hostname":"lima-captain","status":"Stopped","dir":"/private/tmp/clsample/captain","vmType":"vz","arch":"aarch64","cpus":2,"memory":4294967296,"disk":21474836480,"sshConfigFile":"/private/tmp/clsample/captain/ssh.config","sshAddress":"127.0.0.1","protected":false,"limaVersion":"2.2.0","param":{"internal_netplanOptional":"true"},"HostOS":"darwin","HostArch":"aarch64","LimaHome":"/private/tmp/clsample","IdentityFile":"/private/tmp/clsample/_config/user"}
{"name":"broken","status":"Broken","dir":"/x/broken","errors":["open lima.yaml: no such file or directory"]}
"#;

#[test]
fn finds_captain_among_other_instances() {
    let instance = find_instance(SAMPLE, "captain").unwrap();
    assert_eq!(instance.status, "Stopped");
    assert_eq!(instance.dir, "/private/tmp/clsample/captain");
    let resources = instance.resources();
    assert_eq!(resources.cpus, 2);
    assert_eq!(resources.memory_bytes, 4 * GIB);
    assert_eq!(resources.disk_bytes, 20 * GIB);
}

#[test]
fn empty_output_means_no_instance() {
    assert_eq!(find_instance("", "captain"), None);
    assert_eq!(find_instance("not json\n", "captain"), None);
}

#[test]
fn maps_lima_statuses() {
    let status = |name| find_instance(SAMPLE, name).unwrap().host_status();
    assert_eq!(status("default"), HostStatus::Running);
    assert_eq!(status("captain"), HostStatus::Stopped);
    assert_eq!(
        status("broken"),
        HostStatus::Failed("open lima.yaml: no such file or directory".into())
    );
}

#[test]
fn installing_is_starting_and_unknown_is_failed() {
    let line = |status: &str| format!(r#"{{"name":"captain","status":"{status}"}}"#);
    let status = |s: &str| find_instance(&line(s), "captain").unwrap().host_status();
    assert_eq!(status("Installing"), HostStatus::Starting);
    assert!(matches!(status(""), HostStatus::Failed(_)));
    assert!(matches!(status("Broken"), HostStatus::Failed(_)));
}
