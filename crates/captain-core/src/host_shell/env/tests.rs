use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use super::{ShellEnvInput, shell_env};

#[test]
fn points_docker_at_the_engine_and_puts_the_tools_first() {
    let bin = Path::new("/home/me/.captain/bin");
    let path = std::env::join_paths([Path::new("/usr/bin"), bin, Path::new("/bin")]).unwrap();
    let env = shell_env(&ShellEnvInput {
        docker_host: Some("unix:///home/me/.captain/lima/captain/sock/docker.sock"),
        tool_bin: Some(bin),
        path: Some(&path),
        docker_config: Some(Path::new("/home/me/.captain/docker")),
    });
    assert_eq!(
        env.get("DOCKER_HOST"),
        Some(OsStr::new(
            "unix:///home/me/.captain/lima/captain/sock/docker.sock"
        ))
    );
    assert_eq!(env.remove, ["DOCKER_CONTEXT"]);
    let dirs: Vec<PathBuf> = std::env::split_paths(env.get("PATH").unwrap()).collect();
    assert_eq!(dirs, [bin, Path::new("/usr/bin"), Path::new("/bin")]);
    assert_eq!(env.get("TERM"), Some(OsStr::new("xterm-256color")));
    assert_eq!(env.get("TERM_PROGRAM"), Some(OsStr::new("Captain")));
    assert_eq!(
        env.get("DOCKER_CONFIG"),
        Some(Path::new("/home/me/.captain/docker").as_os_str())
    );
    assert_eq!(env.get("KUBECONFIG"), None);

    let unknown = shell_env(&ShellEnvInput::default());
    assert_eq!(unknown.get("DOCKER_HOST"), None);
    assert!(unknown.remove.is_empty());
}
