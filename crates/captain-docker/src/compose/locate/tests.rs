use std::ffi::OsString;
use std::path::{Path, PathBuf};

use super::{BINARY, locate_docker};

fn path_list(dirs: &[&str]) -> OsString {
    std::env::join_paths(dirs).expect("join paths")
}

#[test]
fn path_comes_first() {
    let path = path_list(&["/nowhere", "/custom/bin"]);
    let wanted = Path::new("/custom/bin").join(BINARY);
    let found = locate_docker(Some(&path), None, |p| {
        p == wanted || p.starts_with("/usr/local/bin")
    });
    assert_eq!(found, Some(wanted));
}

#[test]
fn falls_back_to_the_home_install_folders() {
    let home = PathBuf::from("/home/dan");
    let wanted = home.join(".docker/bin").join(BINARY);
    let found = locate_docker(Some(&path_list(&["/nowhere"])), Some(&home), |p| {
        p == wanted
    });
    assert_eq!(found, Some(wanted));
}

#[test]
fn none_when_no_binary_exists() {
    assert_eq!(locate_docker(None, None, |_| false), None);
}
