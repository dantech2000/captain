use std::fs;
use std::path::{Path, PathBuf};

use super::{config_dir, current_host, meta_dir_name, parse_current, parse_meta, read_contexts};

/// A throwaway Docker config dir under the system temp dir.
struct DockerDir(PathBuf);

impl DockerDir {
    fn new(test: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("captain-ctx-{test}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }

    fn write(&self, relative: &str, contents: &str) {
        let path = self.0.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }

    /// Stores a context the way the Docker CLI does, in its hash directory.
    fn context(&self, name: &str, host: &str) {
        let meta = format!(
            r#"{{"Name":"{name}","Metadata":{{"Description":"{name} engine"}},"Endpoints":{{"docker":{{"Host":"{host}","SkipTLSVerify":false}}}}}}"#
        );
        self.write(
            &format!("contexts/meta/{}/meta.json", meta_dir_name(name)),
            &meta,
        );
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for DockerDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn meta_dir_is_sha256_of_name() {
    // `printf rancher-desktop | shasum -a 256`, as in a real ~/.docker.
    assert_eq!(
        meta_dir_name("rancher-desktop"),
        "b547d66a5de60e5f0843aba28283a8875c2ad72e99ba076060ef9ec7c09917c8"
    );
}

#[test]
fn parses_meta_json() {
    let meta = br#"{"Name":"desktop-linux","Metadata":{"Description":"Docker Desktop","otel":{}},
        "Endpoints":{"docker":{"Host":"unix:///Users/me/.docker/run/docker.sock","SkipTLSVerify":false}}}"#;
    let context = parse_meta(meta).unwrap();
    assert_eq!(context.name, "desktop-linux");
    assert_eq!(context.description, "Docker Desktop");
    assert_eq!(
        context.host.as_deref(),
        Some("unix:///Users/me/.docker/run/docker.sock")
    );
    assert_eq!(parse_meta(br#"{"Metadata":{}}"#), None);
}

#[test]
fn default_current_context_is_none() {
    assert_eq!(parse_current(br#"{"currentContext":"default"}"#), None);
    assert_eq!(parse_current(br#"{"auths":{}}"#), None);
    assert_eq!(
        parse_current(br#"{"currentContext":"colima"}"#).as_deref(),
        Some("colima")
    );
}

#[test]
fn lists_contexts_sorted_with_current() {
    let dir = DockerDir::new("list");
    dir.context("colima", "unix:///c.sock");
    dir.context("captain", "unix:///captain.sock");
    dir.write("contexts/meta/junk/meta.json", "not json");
    dir.write("config.json", r#"{"currentContext":"colima"}"#);

    let list = read_contexts(dir.path());
    let names: Vec<_> = list.contexts.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(names, ["captain", "colima"]);
    assert!(list.is_current("colima"));
}

#[test]
fn current_host_follows_override_then_config() {
    let dir = DockerDir::new("current");
    dir.context("colima", "unix:///c.sock");
    dir.context("remote", "ssh://me@box");
    dir.write("config.json", r#"{"currentContext":"colima"}"#);

    assert_eq!(
        current_host(dir.path(), None).as_deref(),
        Some("unix:///c.sock")
    );
    assert_eq!(
        current_host(dir.path(), Some("remote")).as_deref(),
        Some("ssh://me@box")
    );
    assert_eq!(current_host(dir.path(), Some("default")), None);
}

#[test]
fn docker_config_wins_over_home() {
    let home = Path::new("/home/me");
    assert_eq!(
        config_dir(Some("/cfg".into()), Some(home)),
        Some(PathBuf::from("/cfg"))
    );
    assert_eq!(config_dir(None, Some(home)), Some(home.join(".docker")));
}
