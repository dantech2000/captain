use std::fs;
use std::path::{Path, PathBuf};

use super::current_context_host;

/// A throwaway `~/.docker` directory under the system temp dir.
struct DockerDir(PathBuf);

impl DockerDir {
    fn new(test: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("captain-{test}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }

    fn write(&self, relative: &str, contents: &str) {
        let path = self.0.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
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

const COLIMA_META: &str =
    r#"{"Name":"colima","Endpoints":{"docker":{"Host":"unix:///Users/me/.colima/docker.sock"}}}"#;

#[test]
fn reads_host_of_current_context() {
    let dir = DockerDir::new("current");
    dir.write("config.json", r#"{"currentContext":"colima"}"#);
    dir.write("contexts/meta/abc123/meta.json", COLIMA_META);

    assert_eq!(
        current_context_host(dir.path(), None).as_deref(),
        Some("unix:///Users/me/.colima/docker.sock")
    );
}

#[test]
fn override_wins_over_config() {
    let dir = DockerDir::new("override");
    dir.write("config.json", r#"{"currentContext":"other"}"#);
    dir.write("contexts/meta/abc123/meta.json", COLIMA_META);

    assert!(current_context_host(dir.path(), Some("colima")).is_some());
}

#[test]
fn default_context_has_no_host() {
    let dir = DockerDir::new("default");
    dir.write("config.json", r#"{"currentContext":"default"}"#);

    assert_eq!(current_context_host(dir.path(), None), None);
}

#[test]
fn missing_config_has_no_host() {
    let dir = DockerDir::new("missing");

    assert_eq!(current_context_host(dir.path(), None), None);
}
