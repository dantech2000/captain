use std::path::{Path, PathBuf};

use super::LimaPaths;

#[test]
fn lives_in_a_short_dot_folder() {
    let paths = LimaPaths::for_home(Path::new("/Users/ada"));
    assert_eq!(paths.lima_home, PathBuf::from("/Users/ada/.captain/lima"));
    assert_eq!(
        paths.docker_socket(),
        PathBuf::from("/Users/ada/.captain/lima/captain/sock/docker.sock")
    );
    assert_eq!(
        paths.template_file(),
        PathBuf::from("/Users/ada/.captain/captain-engine.yaml")
    );
    assert_eq!(
        paths.lock_file(),
        PathBuf::from("/Users/ada/.captain/captain-engine.lock")
    );
    assert_eq!(
        paths.snapshots_dir(),
        PathBuf::from("/Users/ada/.captain/snapshots")
    );
}

#[test]
fn a_32_character_user_name_fits_but_not_under_application_support() {
    let home = format!("/Users/{}", "a".repeat(32));
    let paths = LimaPaths::for_home(Path::new(&home));
    assert_eq!(paths.check_socket_paths(), Ok(()));
    assert!(paths.docker_socket().as_os_str().len() < 104);
    let home = format!(
        "/Users/{}/Library/Application Support/Captain",
        "a".repeat(32)
    );
    let paths = LimaPaths {
        lima_home: Path::new(&home).join("lima"),
        instance: "captain".into(),
    };
    let error = paths.check_socket_paths().unwrap_err();
    assert!(error.contains("bytes long"), "{error}");
}
